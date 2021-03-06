use crate::{
    application::Application,
    interactive::InteractionsEngine,
    messenger::MessageData,
    widget::{
        component::interactive::navigation::{NavDirection, NavJump, NavSignal, NavType},
        unit::WidgetUnit,
        utils::Vec2,
        WidgetId,
    },
    Scalar,
};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PointerButton {
    Trigger,
    Context,
}

#[derive(Debug, Clone)]
pub enum Interaction {
    None,
    Navigate(NavSignal),
    PointerDown(PointerButton, Scalar, Scalar),
    PointerUp(PointerButton, Scalar, Scalar),
    PointerMove(Scalar, Scalar),
}

impl Default for Interaction {
    fn default() -> Self {
        Self::None
    }
}

impl Interaction {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    #[inline]
    pub fn is_some(&self) -> bool {
        !self.is_none()
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct DefaultInteractionsEngineResult {
    pub captured_pointer_location: bool,
    pub captured_pointer_action: bool,
    pub captured_text_change: bool,
}

impl DefaultInteractionsEngineResult {
    #[inline]
    pub fn is_any(&self) -> bool {
        self.captured_pointer_action || self.captured_pointer_location || self.captured_text_change
    }

    #[inline]
    pub fn is_none(&self) -> bool {
        !self.is_any()
    }
}

/// Single pointer + Keyboard + Gamepad
#[derive(Debug, Default)]
pub struct DefaultInteractionsEngine {
    pub deselect_when_no_button_found: bool,
    interactions_queue: VecDeque<Interaction>,
    containers: HashMap<WidgetId, HashSet<WidgetId>>,
    items_owners: HashMap<WidgetId, WidgetId>,
    buttons: HashSet<WidgetId>,
    text_inputs: HashSet<WidgetId>,
    selected_chain: Vec<WidgetId>,
    focused_text_input: Option<WidgetId>,
    sorted_items_ids: Vec<WidgetId>,
}

impl DefaultInteractionsEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(
        interactions_queue: usize,
        containers: usize,
        buttons: usize,
        text_inputs: usize,
        selected_chain: usize,
    ) -> Self {
        Self {
            deselect_when_no_button_found: false,
            interactions_queue: VecDeque::with_capacity(interactions_queue),
            containers: HashMap::with_capacity(containers),
            items_owners: Default::default(),
            buttons: HashSet::with_capacity(buttons),
            text_inputs: HashSet::with_capacity(text_inputs),
            selected_chain: Vec::with_capacity(selected_chain),
            focused_text_input: None,
            sorted_items_ids: vec![],
        }
    }

    pub fn selected_chain(&self) -> &[WidgetId] {
        &self.selected_chain
    }

    pub fn selected_item(&self) -> Option<&WidgetId> {
        self.selected_chain.last()
    }

    pub fn selected_container(&self) -> Option<&WidgetId> {
        self.selected_chain
            .iter()
            .rev()
            .find(|id| self.containers.contains_key(id))
    }

    pub fn selected_button(&self) -> Option<&WidgetId> {
        self.selected_chain
            .iter()
            .rev()
            .find(|id| self.buttons.contains(id))
    }

    pub fn focused_text_input(&self) -> Option<&WidgetId> {
        self.focused_text_input.as_ref()
    }

    pub fn interact(&mut self, interaction: Interaction) {
        if interaction.is_some() {
            self.interactions_queue.push_back(interaction);
        }
    }

    pub fn clear_queue(&mut self, put_unselect: bool) {
        self.interactions_queue.clear();
        if put_unselect {
            self.interactions_queue
                .push_back(Interaction::Navigate(NavSignal::Unselect));
        }
    }

    fn cache_sorted_items_ids(&mut self, app: &Application) {
        self.sorted_items_ids = Vec::with_capacity(self.items_owners.len());
        self.cache_sorted_items_ids_inner(app.rendered_tree());
    }

    fn cache_sorted_items_ids_inner(&mut self, unit: &WidgetUnit) {
        if let Some(data) = unit.as_data() {
            self.sorted_items_ids.push(data.id().to_owned());
        }
        match unit {
            WidgetUnit::AreaBox(unit) => {
                self.cache_sorted_items_ids_inner(&unit.slot);
            }
            WidgetUnit::ContentBox(unit) => {
                for item in &unit.items {
                    self.cache_sorted_items_ids_inner(&item.slot);
                }
            }
            WidgetUnit::FlexBox(unit) => {
                if unit.direction.is_order_ascending() {
                    for item in &unit.items {
                        self.cache_sorted_items_ids_inner(&item.slot);
                    }
                } else {
                    for item in unit.items.iter().rev() {
                        self.cache_sorted_items_ids_inner(&item.slot);
                    }
                }
            }
            WidgetUnit::GridBox(unit) => {
                for item in &unit.items {
                    self.cache_sorted_items_ids_inner(&item.slot);
                }
            }
            WidgetUnit::SizeBox(unit) => {
                self.cache_sorted_items_ids_inner(&unit.slot);
            }
            _ => {}
        }
    }

    fn select_item(&mut self, app: &mut Application, id: Option<WidgetId>) {
        if self.selected_chain.last() == id.as_ref() {
            return;
        }
        match (self.selected_chain.is_empty(), id) {
            (false, None) => {
                for id in std::mem::take(&mut self.selected_chain).iter().rev() {
                    app.send_message(id, NavSignal::Unselect);
                }
            }
            (false, Some(mut id)) => {
                let mut chain = Vec::with_capacity(self.selected_chain.len());
                while let Some(owner) = self.items_owners.get(&id) {
                    chain.push(id.to_owned());
                    if !chain.contains(owner) {
                        chain.push(owner.to_owned());
                    }
                    id = owner.to_owned();
                }
                chain.reverse();
                let mut index = 0;
                for (a, b) in self.selected_chain.iter().zip(chain.iter()) {
                    if a != b {
                        break;
                    }
                    index += 1;
                }
                for id in &self.selected_chain[index..] {
                    app.send_message(id, NavSignal::Unselect);
                }
                for id in &chain[index..] {
                    app.send_message(id, NavSignal::Select(().into()));
                }
                self.selected_chain = chain;
            }
            (true, Some(mut id)) => {
                self.selected_chain.clear();
                while let Some(owner) = self.items_owners.get(&id) {
                    self.selected_chain.push(id.to_owned());
                    if !self.selected_chain.contains(owner) {
                        self.selected_chain.push(owner.to_owned());
                    }
                    id = owner.to_owned();
                }
                self.selected_chain.reverse();
                for id in &self.selected_chain {
                    app.send_message(id, NavSignal::Select(().into()));
                }
            }
            _ => {}
        }
    }

    fn focus_text_input(&mut self, app: &mut Application, id: Option<WidgetId>) {
        if self.focused_text_input == id {
            return;
        }
        if let Some(focused) = &self.focused_text_input {
            app.send_message(focused, NavSignal::FocusTextInput(().into()));
        }
        self.focused_text_input = None;
        if let Some(id) = id {
            if self.text_inputs.contains(&id) {
                app.send_message(&id, NavSignal::FocusTextInput(id.to_owned().into()));
                self.focused_text_input = Some(id);
            }
        }
    }

    fn send_to_selected_item<T>(&self, app: &mut Application, data: T) -> bool
    where
        T: 'static + MessageData,
    {
        if let Some(id) = self.selected_item() {
            app.send_message(id, data);
            return true;
        }
        false
    }

    fn send_to_focused_text_input<T>(&self, app: &mut Application, data: T) -> bool
    where
        T: 'static + MessageData,
    {
        if let Some(id) = self.focused_text_input() {
            app.send_message(id, data);
            return true;
        }
        false
    }

    fn send_to_selected_container<T>(&self, app: &mut Application, data: T) -> bool
    where
        T: 'static + MessageData,
    {
        if let Some(id) = self.selected_container() {
            app.send_message(id, data);
            return true;
        }
        false
    }

    fn send_to_selected_button<T>(&self, app: &mut Application, data: T) -> bool
    where
        T: 'static + MessageData,
    {
        if let Some(id) = self.selected_button() {
            app.send_message(id, data);
            return true;
        }
        false
    }

    fn get_item_point(app: &Application, id: &WidgetId) -> Option<Vec2> {
        if let Some(layout) = app.layout_data().items.get(id) {
            let x = (layout.ui_space.left + layout.ui_space.right) * 0.5;
            let y = (layout.ui_space.top + layout.ui_space.bottom) * 0.5;
            Some(Vec2 { x, y })
        } else {
            None
        }
    }

    fn find_item_closest_to_point(
        app: &Application,
        point: Vec2,
        items: &HashSet<WidgetId>,
    ) -> Option<WidgetId> {
        items
            .iter()
            .filter_map(|id| {
                Self::get_item_point(app, id).map(|p| {
                    let dx = p.x - point.x;
                    let dy = p.y - point.y;
                    (id, dx * dx + dy * dy)
                })
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|m| m.0.to_owned())
    }

    fn find_item_closest_to_direction(
        app: &Application,
        point: Vec2,
        direction: NavDirection,
        items: &HashSet<WidgetId>,
    ) -> Option<WidgetId> {
        let dir = match direction {
            NavDirection::Up => Vec2 { x: 0.0, y: -1.0 },
            NavDirection::Down => Vec2 { x: 0.0, y: 1.0 },
            NavDirection::Left => Vec2 { x: -1.0, y: 0.0 },
            NavDirection::Right => Vec2 { x: 1.0, y: 0.0 },
            _ => return None,
        };
        items
            .iter()
            .filter_map(|id| {
                Self::get_item_point(app, id).map(|p| {
                    let dx = p.x - point.x;
                    let dy = p.y - point.y;
                    let dot = dx * dir.x + dy * dir.y;
                    let len = (dx * dx + dy * dy).sqrt();
                    let f = if len > 0.0 { dot / len } else { dot };
                    (id, f)
                })
            })
            .filter(|m| m.1 > 1.0e-6)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|m| m.0.to_owned())
    }

    fn find_first_item(&self, items: &HashSet<WidgetId>) -> Option<WidgetId> {
        self.sorted_items_ids
            .iter()
            .find(|id| items.contains(id))
            .cloned()
    }

    fn find_last_item(&self, items: &HashSet<WidgetId>) -> Option<WidgetId> {
        self.sorted_items_ids
            .iter()
            .rev()
            .find(|id| items.contains(id))
            .cloned()
    }

    fn find_prev_item(&self, id: &WidgetId, items: &HashSet<WidgetId>) -> Option<WidgetId> {
        let mut found = false;
        self.sorted_items_ids
            .iter()
            .rev()
            .find(|i| {
                if found {
                    if items.contains(i) {
                        return true;
                    }
                } else if i == &id {
                    found = true;
                }
                false
            })
            .cloned()
    }

    fn find_next_item(&self, id: &WidgetId, items: &HashSet<WidgetId>) -> Option<WidgetId> {
        let mut found = false;
        self.sorted_items_ids
            .iter()
            .find(|i| {
                if found {
                    if items.contains(i) {
                        return true;
                    }
                } else if i == &id {
                    found = true;
                }
                false
            })
            .cloned()
    }

    fn jump(&mut self, app: &mut Application, id: &WidgetId, data: NavJump) {
        if let Some(items) = self.containers.get(id) {
            match data {
                NavJump::First => {
                    if let Some(id) = self.find_first_item(items) {
                        self.select_item(app, Some(id));
                    }
                }
                NavJump::Last => {
                    if let Some(id) = self.find_last_item(items) {
                        self.select_item(app, Some(id));
                    }
                }
                NavJump::TopLeft => {
                    if let Some(layout) = app.layout_data().items.get(id) {
                        let point = Vec2 {
                            x: layout.ui_space.left,
                            y: layout.ui_space.top,
                        };
                        if let Some(id) = Self::find_item_closest_to_point(app, point, items) {
                            self.select_item(app, Some(id));
                        }
                    }
                }
                NavJump::TopRight => {
                    if let Some(layout) = app.layout_data().items.get(id) {
                        let point = Vec2 {
                            x: layout.ui_space.right,
                            y: layout.ui_space.top,
                        };
                        if let Some(id) = Self::find_item_closest_to_point(app, point, items) {
                            self.select_item(app, Some(id));
                        }
                    }
                }
                NavJump::BottomLeft => {
                    if let Some(layout) = app.layout_data().items.get(id) {
                        let point = Vec2 {
                            x: layout.ui_space.left,
                            y: layout.ui_space.bottom,
                        };
                        if let Some(id) = Self::find_item_closest_to_point(app, point, items) {
                            self.select_item(app, Some(id));
                        }
                    }
                }
                NavJump::BottomRight => {
                    if let Some(layout) = app.layout_data().items.get(id) {
                        let point = Vec2 {
                            x: layout.ui_space.right,
                            y: layout.ui_space.bottom,
                        };
                        if let Some(id) = Self::find_item_closest_to_point(app, point, items) {
                            self.select_item(app, Some(id));
                        }
                    }
                }
                NavJump::Loop(direction) => match direction {
                    NavDirection::Up
                    | NavDirection::Down
                    | NavDirection::Left
                    | NavDirection::Right => {
                        if let Some(point) = Self::get_item_point(app, id) {
                            if let Some(id) =
                                Self::find_item_closest_to_direction(app, point, direction, items)
                            {
                                self.select_item(app, Some(id));
                            }
                        }
                    }
                    NavDirection::Prev => {
                        if let Some(id) = self.selected_chain.last() {
                            if let Some(id) = self.find_prev_item(id, items) {
                                self.select_item(app, Some(id));
                            } else if let Some(id) = self.find_last_item(items) {
                                self.select_item(app, Some(id));
                            }
                        }
                    }
                    NavDirection::Next => {
                        if let Some(id) = self.selected_chain.last() {
                            if let Some(id) = self.find_next_item(id, items) {
                                self.select_item(app, Some(id));
                            } else if let Some(id) = self.find_first_item(items) {
                                self.select_item(app, Some(id));
                            }
                        }
                    }
                    _ => {}
                },
                NavJump::Escape(direction, idref) => match direction {
                    NavDirection::Up
                    | NavDirection::Down
                    | NavDirection::Left
                    | NavDirection::Right => {
                        if let Some(point) = Self::get_item_point(app, id) {
                            if let Some(id) =
                                Self::find_item_closest_to_direction(app, point, direction, items)
                            {
                                self.select_item(app, Some(id));
                            } else if let Some(id) = idref.read() {
                                self.select_item(app, Some(id));
                            }
                        }
                    }
                    NavDirection::Prev => {
                        if let Some(id) = self.selected_chain.last() {
                            if let Some(id) = self.find_prev_item(id, items) {
                                self.select_item(app, Some(id));
                            } else if let Some(id) = idref.read() {
                                self.select_item(app, Some(id));
                            }
                        }
                    }
                    NavDirection::Next => {
                        if let Some(id) = self.selected_chain.last() {
                            if let Some(id) = self.find_next_item(id, items) {
                                self.select_item(app, Some(id));
                            } else if let Some(id) = idref.read() {
                                self.select_item(app, Some(id));
                            }
                        }
                    }
                    _ => {}
                },
            }
        }
    }

    fn find_button(&self, app: &Application, x: Scalar, y: Scalar) -> Option<WidgetId> {
        self.find_button_inner(app, x, y, app.rendered_tree())
    }

    fn find_button_inner(
        &self,
        app: &Application,
        x: Scalar,
        y: Scalar,
        unit: &WidgetUnit,
    ) -> Option<WidgetId> {
        let mut result = None;
        if let Some(data) = unit.as_data() {
            if self.buttons.contains(data.id()) {
                if let Some(layout) = app.layout_data().items.get(data.id()) {
                    let rect = layout.ui_space;
                    if x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom {
                        result = Some(data.id().to_owned());
                    }
                }
            }
        }
        match unit {
            WidgetUnit::AreaBox(unit) => {
                if let Some(id) = self.find_button_inner(app, x, y, &unit.slot) {
                    result = Some(id);
                }
            }
            WidgetUnit::ContentBox(unit) => {
                for item in &unit.items {
                    if let Some(id) = self.find_button_inner(app, x, y, &item.slot) {
                        result = Some(id);
                    }
                }
            }
            WidgetUnit::FlexBox(unit) => {
                for item in &unit.items {
                    if let Some(id) = self.find_button_inner(app, x, y, &item.slot) {
                        result = Some(id);
                    }
                }
            }
            WidgetUnit::GridBox(unit) => {
                for item in &unit.items {
                    if let Some(id) = self.find_button_inner(app, x, y, &item.slot) {
                        result = Some(id);
                    }
                }
            }
            WidgetUnit::SizeBox(unit) => {
                if let Some(id) = self.find_button_inner(app, x, y, &unit.slot) {
                    result = Some(id);
                }
            }
            _ => {}
        }
        result
    }

    fn does_hover_widget(&self, app: &Application, x: Scalar, y: Scalar) -> bool {
        self.does_hover_widget_inner(app, x, y, app.rendered_tree())
    }

    fn does_hover_widget_inner(
        &self,
        app: &Application,
        x: Scalar,
        y: Scalar,
        unit: &WidgetUnit,
    ) -> bool {
        if let Some(data) = unit.as_data() {
            if let Some(layout) = app.layout_data().items.get(data.id()) {
                let rect = layout.ui_space;
                if x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom {
                    return true;
                }
            }
        }
        match unit {
            WidgetUnit::AreaBox(unit) => {
                if self.does_hover_widget_inner(app, x, y, &unit.slot) {
                    return true;
                }
            }
            WidgetUnit::ContentBox(unit) => {
                for item in &unit.items {
                    if self.does_hover_widget_inner(app, x, y, &item.slot) {
                        return true;
                    }
                }
            }
            WidgetUnit::FlexBox(unit) => {
                for item in &unit.items {
                    if self.does_hover_widget_inner(app, x, y, &item.slot) {
                        return true;
                    }
                }
            }
            WidgetUnit::GridBox(unit) => {
                for item in &unit.items {
                    if self.does_hover_widget_inner(app, x, y, &item.slot) {
                        return true;
                    }
                }
            }
            WidgetUnit::SizeBox(unit) => {
                if self.does_hover_widget_inner(app, x, y, &unit.slot) {
                    return true;
                }
            }
            _ => {}
        }
        false
    }
}

impl InteractionsEngine<DefaultInteractionsEngineResult, ()> for DefaultInteractionsEngine {
    fn perform_interactions(
        &mut self,
        app: &mut Application,
    ) -> Result<DefaultInteractionsEngineResult, ()> {
        let mut to_select = None;
        let mut to_jump = HashMap::new();
        let mut to_focus = None;
        let mut to_send_axis = vec![];
        let mut to_send_custom = vec![];
        for (id, signal) in app.signals() {
            if let Some(signal) = signal.as_any().downcast_ref::<NavSignal>() {
                match signal {
                    NavSignal::Register(t) => match t {
                        NavType::Container => {
                            self.containers.insert(id.to_owned(), Default::default());
                        }
                        NavType::Item => {
                            if let Some((key, items)) = self
                                .containers
                                .iter_mut()
                                .filter(|(k, _)| id.path().starts_with(k.path()))
                                .max_by(|(a, _), (b, _)| a.depth().cmp(&b.depth()))
                            {
                                items.remove(id);
                                items.insert(id.to_owned());
                                self.items_owners.insert(id.to_owned(), key.to_owned());
                            }
                        }
                        NavType::Button => {
                            self.buttons.insert(id.to_owned());
                        }
                        NavType::TextInput => {
                            self.text_inputs.insert(id.to_owned());
                        }
                    },
                    NavSignal::Unregister(t) => match t {
                        NavType::Container => {
                            if let Some(items) = self.containers.remove(id) {
                                for id in items {
                                    self.items_owners.remove(&id);
                                }
                            }
                        }
                        NavType::Item => {
                            if let Some(key) = self.items_owners.remove(id) {
                                if let Some(items) = self.containers.get_mut(&key) {
                                    items.remove(&key);
                                }
                            }
                        }
                        NavType::Button => {
                            self.buttons.remove(id);
                        }
                        NavType::TextInput => {
                            self.text_inputs.remove(id);
                            if let Some(focused) = &self.focused_text_input {
                                if focused == id {
                                    self.focused_text_input = None;
                                }
                            }
                        }
                    },
                    NavSignal::Select(idref) => to_select = Some(idref.to_owned()),
                    NavSignal::Unselect => to_select = Some(().into()),
                    NavSignal::Jump(data) => {
                        to_jump.insert(id.to_owned(), data.to_owned());
                    }
                    NavSignal::FocusTextInput(idref) => to_focus = Some(idref.to_owned()),
                    NavSignal::Axis(name, value) => to_send_axis.push((name.to_owned(), *value)),
                    NavSignal::Custom(idref, data) => {
                        to_send_custom.push((idref.to_owned(), data.to_owned()))
                    }
                    _ => {}
                }
            }
        }
        if let Some(idref) = to_select {
            self.select_item(app, idref.read());
        }
        if !to_jump.is_empty() {
            self.cache_sorted_items_ids(app);
        }
        for (id, data) in to_jump {
            self.jump(app, &id, data);
        }
        if let Some(idref) = to_focus {
            self.focus_text_input(app, idref.read());
        }
        for (name, value) in to_send_axis {
            self.send_to_selected_item(app, NavSignal::Axis(name, value));
        }
        for (idref, data) in to_send_custom {
            if let Some(id) = idref.read() {
                app.send_message(&id, NavSignal::Custom(().into(), data));
            } else {
                self.send_to_selected_item(app, NavSignal::Custom(().into(), data));
            }
        }
        let mut result = DefaultInteractionsEngineResult::default();
        while let Some(interaction) = self.interactions_queue.pop_front() {
            match interaction {
                Interaction::None => {}
                Interaction::Navigate(msg) => match msg {
                    NavSignal::Select(idref) => self.select_item(app, idref.read()),
                    NavSignal::Unselect => self.select_item(app, None),
                    NavSignal::Accept(_) | NavSignal::Context(_) | NavSignal::Cancel(_) => {
                        self.send_to_selected_item(app, msg);
                    }
                    NavSignal::Up
                    | NavSignal::Down
                    | NavSignal::Left
                    | NavSignal::Right
                    | NavSignal::Prev
                    | NavSignal::Next => {
                        self.send_to_selected_container(app, msg);
                    }
                    NavSignal::FocusTextInput(idref) => {
                        self.focus_text_input(app, idref.read());
                    }
                    NavSignal::TextChange(_) => {
                        if self.send_to_focused_text_input(app, msg) {
                            result.captured_text_change = true;
                        }
                    }
                    NavSignal::Custom(idref, data) => {
                        if let Some(id) = idref.read() {
                            app.send_message(&id, NavSignal::Custom(().into(), data));
                        } else {
                            self.send_to_selected_item(app, NavSignal::Custom(().into(), data));
                        }
                    }
                    _ => {}
                },
                Interaction::PointerMove(x, y) => {
                    let found = self.find_button(app, x, y);
                    if found.is_some() {
                        self.select_item(app, found);
                        result.captured_pointer_location = true;
                    } else {
                        if self.deselect_when_no_button_found {
                            self.select_item(app, None);
                        }
                        if self.does_hover_widget(app, x, y) {
                            result.captured_pointer_location = true;
                        }
                    }
                }
                Interaction::PointerDown(button, x, y) => {
                    let found = self.find_button(app, x, y);
                    if found.is_some() {
                        self.select_item(app, found);
                        result.captured_pointer_location = true;
                        let action = match button {
                            PointerButton::Trigger => NavSignal::Accept(true),
                            PointerButton::Context => NavSignal::Context(true),
                        };
                        if self.send_to_selected_button(app, action) {
                            result.captured_pointer_action = true;
                        }
                    } else {
                        if self.deselect_when_no_button_found {
                            self.select_item(app, None);
                        }
                        if self.does_hover_widget(app, x, y) {
                            result.captured_pointer_location = true;
                        }
                    }
                }
                Interaction::PointerUp(button, _, _) => {
                    let action = match button {
                        PointerButton::Trigger => NavSignal::Accept(false),
                        PointerButton::Context => NavSignal::Context(false),
                    };
                    if self.send_to_selected_button(app, action) {
                        result.captured_pointer_action = true;
                    }
                }
            }
        }
        Ok(result)
    }
}
