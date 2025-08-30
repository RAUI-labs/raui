use crate::{
    Scalar,
    application::Application,
    interactive::InteractionsEngine,
    messenger::MessageData,
    widget::{
        WidgetId,
        component::{
            RelativeLayoutListenerSignal, ResizeListenerSignal,
            interactive::navigation::{NavDirection, NavJump, NavScroll, NavSignal, NavType},
        },
        unit::WidgetUnit,
        utils::{Rect, Vec2, lerp},
    },
};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PointerButton {
    Trigger,
    Context,
}

#[derive(Debug, Default, Clone)]
pub enum Interaction {
    #[default]
    None,
    Navigate(NavSignal),
    PointerDown(PointerButton, Vec2),
    PointerUp(PointerButton, Vec2),
    PointerMove(Vec2),
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
    pub unfocus_when_selection_change: bool,
    resize_listeners: HashMap<WidgetId, Vec2>,
    relative_layout_listeners: HashMap<WidgetId, (WidgetId, Vec2, Rect)>,
    interactions_queue: VecDeque<Interaction>,
    containers: HashMap<WidgetId, HashSet<WidgetId>>,
    items_owners: HashMap<WidgetId, WidgetId>,
    buttons: HashSet<WidgetId>,
    text_inputs: HashSet<WidgetId>,
    scroll_views: HashSet<WidgetId>,
    scroll_view_contents: HashSet<WidgetId>,
    tracking: HashMap<WidgetId, WidgetId>,
    selected_chain: Vec<WidgetId>,
    locked_widget: Option<WidgetId>,
    focused_text_input: Option<WidgetId>,
    sorted_items_ids: Vec<WidgetId>,
}

impl DefaultInteractionsEngine {
    #[allow(clippy::too_many_arguments)]
    pub fn with_capacity(
        resize_listeners: usize,
        relative_layout_listeners: usize,
        interactions_queue: usize,
        containers: usize,
        buttons: usize,
        text_inputs: usize,
        scroll_views: usize,
        tracking: usize,
        selected_chain: usize,
    ) -> Self {
        Self {
            deselect_when_no_button_found: false,
            unfocus_when_selection_change: true,
            resize_listeners: HashMap::with_capacity(resize_listeners),
            relative_layout_listeners: HashMap::with_capacity(relative_layout_listeners),
            interactions_queue: VecDeque::with_capacity(interactions_queue),
            containers: HashMap::with_capacity(containers),
            items_owners: Default::default(),
            buttons: HashSet::with_capacity(buttons),
            text_inputs: HashSet::with_capacity(text_inputs),
            scroll_views: HashSet::with_capacity(scroll_views),
            scroll_view_contents: HashSet::with_capacity(scroll_views),
            tracking: HashMap::with_capacity(tracking),
            selected_chain: Vec::with_capacity(selected_chain),
            locked_widget: None,
            focused_text_input: None,
            sorted_items_ids: vec![],
        }
    }

    pub fn locked_widget(&self) -> Option<&WidgetId> {
        self.locked_widget.as_ref()
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

    pub fn selected_scroll_view(&self) -> Option<&WidgetId> {
        self.selected_chain
            .iter()
            .rev()
            .find(|id| self.scroll_views.contains(id))
    }

    pub fn selected_scroll_view_content(&self) -> Option<&WidgetId> {
        self.selected_chain
            .iter()
            .rev()
            .find(|id| self.scroll_view_contents.contains(id))
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

    pub fn select_item(&mut self, app: &mut Application, id: Option<WidgetId>) -> bool {
        if self.locked_widget.is_some() || self.selected_chain.last() == id.as_ref() {
            return false;
        }
        if let Some(id) = &id
            && self.containers.contains_key(id)
        {
            app.send_message(id, NavSignal::Select(id.to_owned().into()));
        }
        match (self.selected_chain.is_empty(), id) {
            (false, None) => {
                for id in std::mem::take(&mut self.selected_chain).iter().rev() {
                    app.send_message(id, NavSignal::Unselect);
                }
            }
            (false, Some(mut id)) => {
                if self.unfocus_when_selection_change {
                    self.focus_text_input(app, None);
                }
                let mut chain = Vec::with_capacity(self.selected_chain.len());
                while let Some(owner) = self.items_owners.get(&id) {
                    if !chain.contains(&id) {
                        chain.push(id.to_owned());
                    }
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
                if self.unfocus_when_selection_change {
                    self.focus_text_input(app, None);
                }
                self.selected_chain.clear();
                while let Some(owner) = self.items_owners.get(&id) {
                    if !self.selected_chain.contains(&id) {
                        self.selected_chain.push(id.to_owned());
                    }
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
            (true, None) => {}
        }
        true
    }

    pub fn focus_text_input(&mut self, app: &mut Application, id: Option<WidgetId>) {
        if self.focused_text_input == id {
            return;
        }
        if let Some(focused) = &self.focused_text_input {
            app.send_message(focused, NavSignal::FocusTextInput(().into()));
        }
        self.focused_text_input = None;
        if let Some(id) = id
            && self.text_inputs.contains(&id)
        {
            app.send_message(&id, NavSignal::FocusTextInput(id.to_owned().into()));
            self.focused_text_input = Some(id);
        }
    }

    pub fn send_to_selected_item<T>(&self, app: &mut Application, data: T) -> bool
    where
        T: 'static + MessageData,
    {
        if let Some(id) = self.selected_item() {
            app.send_message(id, data);
            return true;
        }
        false
    }

    pub fn send_to_selected_container<T>(&self, app: &mut Application, data: T) -> bool
    where
        T: 'static + MessageData,
    {
        if let Some(id) = self.selected_container() {
            app.send_message(id, data);
            return true;
        }
        false
    }

    pub fn send_to_selected_button<T>(&self, app: &mut Application, data: T) -> bool
    where
        T: 'static + MessageData,
    {
        if let Some(id) = self.selected_button() {
            app.send_message(id, data);
            return true;
        }
        false
    }

    pub fn send_to_focused_text_input<T>(&self, app: &mut Application, data: T) -> bool
    where
        T: 'static + MessageData,
    {
        if let Some(id) = self.focused_text_input() {
            app.send_message(id, data);
            return true;
        }
        false
    }

    fn find_scroll_view_content(&self, id: &WidgetId) -> Option<WidgetId> {
        if self.scroll_views.contains(id)
            && let Some(items) = self.containers.get(id)
        {
            for item in items {
                if self.scroll_view_contents.contains(item) {
                    return Some(item.to_owned());
                }
            }
        }
        None
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

    fn get_selected_item_point(&self, app: &Application) -> Option<Vec2> {
        Self::get_item_point(app, self.selected_item()?)
    }

    fn get_closest_item_point(app: &Application, id: &WidgetId, mut point: Vec2) -> Option<Vec2> {
        if let Some(layout) = app.layout_data().items.get(id) {
            point.x = point.x.max(layout.ui_space.left).min(layout.ui_space.right);
            point.y = point.y.max(layout.ui_space.top).min(layout.ui_space.bottom);
            Some(point)
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
                Self::get_closest_item_point(app, id, point).map(|p| {
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
                Self::get_closest_item_point(app, id, point).map(|p| {
                    let dx = p.x - point.x;
                    let dy = p.y - point.y;
                    let len = (dx * dx + dy * dy).sqrt();
                    let dot = dx / len * dir.x + dy / len * dir.y;
                    let f = if len > 0.0 { dot / len } else { 0.0 };
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

    // TODO: refactor this shit! my eyes are bleeding, like really dude ffs..
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
                NavJump::MiddleCenter => {
                    if let Some(layout) = app.layout_data().items.get(id) {
                        let point = Vec2 {
                            x: (layout.ui_space.left + layout.ui_space.right) * 0.5,
                            y: (layout.ui_space.top + layout.ui_space.bottom) * 0.5,
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
                        if let Some(point) = self.get_selected_item_point(app) {
                            if let Some(id) =
                                Self::find_item_closest_to_direction(app, point, direction, items)
                            {
                                self.select_item(app, Some(id));
                            } else if let Some(id) = self.items_owners.get(id) {
                                match direction {
                                    NavDirection::Up => app.send_message(id, NavSignal::Up),
                                    NavDirection::Down => app.send_message(id, NavSignal::Down),
                                    NavDirection::Left => app.send_message(id, NavSignal::Left),
                                    NavDirection::Right => app.send_message(id, NavSignal::Right),
                                    _ => {}
                                }
                            }
                        }
                    }
                    NavDirection::Prev => {
                        if let Some(id) = self.selected_chain.last() {
                            if let Some(id) = self.find_prev_item(id, items) {
                                self.select_item(app, Some(id));
                            } else if let Some(id) = self.find_last_item(items) {
                                self.select_item(app, Some(id));
                            } else if let Some(id) = self.items_owners.get(id) {
                                app.send_message(id, NavSignal::Prev);
                            }
                        }
                    }
                    NavDirection::Next => {
                        if let Some(id) = self.selected_chain.last() {
                            if let Some(id) = self.find_next_item(id, items) {
                                self.select_item(app, Some(id));
                            } else if let Some(id) = self.find_first_item(items) {
                                self.select_item(app, Some(id));
                            } else if let Some(id) = self.items_owners.get(id) {
                                app.send_message(id, NavSignal::Next);
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
                        if let Some(point) = self.get_selected_item_point(app) {
                            if let Some(id) =
                                Self::find_item_closest_to_direction(app, point, direction, items)
                            {
                                self.select_item(app, Some(id));
                            } else if let Some(id) = idref.read() {
                                self.select_item(app, Some(id));
                            } else if let Some(id) = self.items_owners.get(id) {
                                match direction {
                                    NavDirection::Up => app.send_message(id, NavSignal::Up),
                                    NavDirection::Down => app.send_message(id, NavSignal::Down),
                                    NavDirection::Left => app.send_message(id, NavSignal::Left),
                                    NavDirection::Right => app.send_message(id, NavSignal::Right),
                                    _ => {}
                                }
                            }
                        }
                    }
                    NavDirection::Prev => {
                        if let Some(id) = self.selected_chain.last() {
                            if let Some(id) = self.find_prev_item(id, items) {
                                self.select_item(app, Some(id));
                            } else if let Some(id) = idref.read() {
                                self.select_item(app, Some(id));
                            } else if let Some(id) = self.items_owners.get(id) {
                                app.send_message(id, NavSignal::Prev);
                            }
                        }
                    }
                    NavDirection::Next => {
                        if let Some(id) = self.selected_chain.last() {
                            if let Some(id) = self.find_next_item(id, items) {
                                self.select_item(app, Some(id));
                            } else if let Some(id) = idref.read() {
                                self.select_item(app, Some(id));
                            } else if let Some(id) = self.items_owners.get(id) {
                                app.send_message(id, NavSignal::Next);
                            }
                        }
                    }
                    _ => {}
                },
                NavJump::Scroll(scroll) => {
                    fn factor(
                        this: &DefaultInteractionsEngine,
                        app: &mut Application,
                        id: &WidgetId,
                        v: Vec2,
                        relative: bool,
                    ) {
                        if let Some(oid) = this.find_scroll_view_content(id) {
                            let a = app.layout_data().find_or_ui_space(oid.path());
                            let b = app.layout_data().find_or_ui_space(id.path());
                            let asize = a.local_space.size();
                            let bsize = b.local_space.size();
                            let f = Vec2 {
                                x: if bsize.x > 0.0 {
                                    asize.x / bsize.x
                                } else {
                                    0.0
                                },
                                y: if bsize.y > 0.0 {
                                    asize.y / bsize.y
                                } else {
                                    0.0
                                },
                            };
                            app.send_message(
                                id,
                                NavSignal::Jump(NavJump::Scroll(NavScroll::Change(v, f, relative))),
                            );
                        }
                    }

                    fn units(
                        this: &DefaultInteractionsEngine,
                        app: &mut Application,
                        id: &WidgetId,
                        v: Vec2,
                        relative: bool,
                    ) {
                        if let Some(oid) = this.find_scroll_view_content(id) {
                            let a = app.layout_data().find_or_ui_space(oid.path());
                            let b = app.layout_data().find_or_ui_space(id.path());
                            let asize = a.local_space.size();
                            let bsize = b.local_space.size();
                            let dsize = Vec2 {
                                x: asize.x - bsize.x,
                                y: asize.y - bsize.y,
                            };
                            let v = Vec2 {
                                x: if dsize.x > 0.0 { v.x / dsize.x } else { 0.0 },
                                y: if dsize.y > 0.0 { v.y / dsize.y } else { 0.0 },
                            };
                            let f = Vec2 {
                                x: if bsize.x > 0.0 {
                                    asize.x / bsize.x
                                } else {
                                    0.0
                                },
                                y: if bsize.y > 0.0 {
                                    asize.y / bsize.y
                                } else {
                                    0.0
                                },
                            };
                            app.send_message(
                                id,
                                NavSignal::Jump(NavJump::Scroll(NavScroll::Change(v, f, relative))),
                            );
                        }
                    }

                    match scroll {
                        NavScroll::Factor(v, relative) => factor(self, app, id, v, relative),
                        NavScroll::DirectFactor(idref, v, relative) => {
                            if let Some(id) = idref.read() {
                                factor(self, app, &id, v, relative);
                            }
                        }
                        NavScroll::Units(v, relative) => units(self, app, id, v, relative),
                        NavScroll::DirectUnits(idref, v, relative) => {
                            if let Some(id) = idref.read() {
                                units(self, app, &id, v, relative);
                            }
                        }
                        NavScroll::Widget(idref, anchor) => {
                            if let (Some(wid), Some(oid)) =
                                (idref.read(), self.find_scroll_view_content(id))
                                && let Some(rect) = app.layout_data().rect_relative_to(&wid, &oid)
                            {
                                let aitem = app.layout_data().find_or_ui_space(oid.path());
                                let bitem = app.layout_data().find_or_ui_space(id.path());
                                let x = lerp(rect.left, rect.right, anchor.x);
                                let y = lerp(rect.top, rect.bottom, anchor.y);
                                let asize = aitem.local_space.size();
                                let bsize = bitem.local_space.size();
                                let v = Vec2 {
                                    x: if asize.x > 0.0 { x / asize.x } else { 0.0 },
                                    y: if asize.y > 0.0 { y / asize.y } else { 0.0 },
                                };
                                let f = Vec2 {
                                    x: if bsize.x > 0.0 {
                                        asize.x / bsize.x
                                    } else {
                                        0.0
                                    },
                                    y: if bsize.y > 0.0 {
                                        asize.y / bsize.y
                                    } else {
                                        0.0
                                    },
                                };
                                app.send_message(
                                    id,
                                    NavSignal::Jump(NavJump::Scroll(NavScroll::Change(
                                        v, f, false,
                                    ))),
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn find_button(&self, app: &Application, x: Scalar, y: Scalar) -> Option<(WidgetId, Vec2)> {
        self.find_button_inner(app, x, y, app.rendered_tree(), app.layout_data().ui_space)
    }

    fn find_button_inner(
        &self,
        app: &Application,
        x: Scalar,
        y: Scalar,
        unit: &WidgetUnit,
        mut clip: Rect,
    ) -> Option<(WidgetId, Vec2)> {
        if x < clip.left || x > clip.right || y < clip.top || y > clip.bottom {
            return None;
        }
        let mut result = None;
        if let Some(data) = unit.as_data()
            && self.buttons.contains(data.id())
            && let Some(layout) = app.layout_data().items.get(data.id())
        {
            let rect = layout.ui_space;
            if x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom {
                let size = rect.size();
                let pos = Vec2 {
                    x: if size.x > 0.0 {
                        (x - rect.left) / size.x
                    } else {
                        0.0
                    },
                    y: if size.y > 0.0 {
                        (y - rect.top) / size.y
                    } else {
                        0.0
                    },
                };
                result = Some((data.id().to_owned(), pos));
            }
        }
        match unit {
            WidgetUnit::AreaBox(unit) => {
                if let Some(id) = self.find_button_inner(app, x, y, &unit.slot, clip) {
                    result = Some(id);
                }
            }
            WidgetUnit::ContentBox(unit) => {
                if unit.clipping
                    && let Some(item) = app.layout_data().items.get(&unit.id)
                {
                    clip = item.ui_space;
                }
                for item in &unit.items {
                    if let Some(id) = self.find_button_inner(app, x, y, &item.slot, clip) {
                        result = Some(id);
                    }
                }
            }
            WidgetUnit::FlexBox(unit) => {
                for item in &unit.items {
                    if let Some(id) = self.find_button_inner(app, x, y, &item.slot, clip) {
                        result = Some(id);
                    }
                }
            }
            WidgetUnit::GridBox(unit) => {
                for item in &unit.items {
                    if let Some(id) = self.find_button_inner(app, x, y, &item.slot, clip) {
                        result = Some(id);
                    }
                }
            }
            WidgetUnit::SizeBox(unit) => {
                if let Some(id) = self.find_button_inner(app, x, y, &unit.slot, clip) {
                    result = Some(id);
                }
            }
            _ => {}
        }
        result
    }

    pub fn does_hover_widget(&self, app: &Application, x: Scalar, y: Scalar) -> bool {
        Self::does_hover_widget_inner(app, x, y, app.rendered_tree())
    }

    fn does_hover_widget_inner(app: &Application, x: Scalar, y: Scalar, unit: &WidgetUnit) -> bool {
        if let Some(data) = unit.as_data()
            && let Some(layout) = app.layout_data().items.get(data.id())
        {
            let rect = layout.ui_space;
            if x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom {
                return true;
            }
        }
        match unit {
            WidgetUnit::AreaBox(unit) => {
                if Self::does_hover_widget_inner(app, x, y, &unit.slot) {
                    return true;
                }
            }
            WidgetUnit::ContentBox(unit) => {
                for item in &unit.items {
                    if Self::does_hover_widget_inner(app, x, y, &item.slot) {
                        return true;
                    }
                }
            }
            WidgetUnit::FlexBox(unit) => {
                for item in &unit.items {
                    if Self::does_hover_widget_inner(app, x, y, &item.slot) {
                        return true;
                    }
                }
            }
            WidgetUnit::GridBox(unit) => {
                for item in &unit.items {
                    if Self::does_hover_widget_inner(app, x, y, &item.slot) {
                        return true;
                    }
                }
            }
            WidgetUnit::SizeBox(unit) => {
                if Self::does_hover_widget_inner(app, x, y, &unit.slot) {
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
        let mut to_resize = HashSet::new();
        let mut to_relative_layout = HashSet::new();
        let mut to_select = None;
        let mut to_jump = HashMap::new();
        let mut to_focus = None;
        let mut to_send_axis = vec![];
        let mut to_send_custom = vec![];
        for (id, signal) in app.signals() {
            if let Some(signal) = signal.as_any().downcast_ref() {
                match signal {
                    ResizeListenerSignal::Register => {
                        if let Some(item) = app.layout_data().items.get(id) {
                            self.resize_listeners
                                .insert(id.to_owned(), item.local_space.size());
                            to_resize.insert(id.to_owned());
                        }
                    }
                    ResizeListenerSignal::Unregister => {
                        self.resize_listeners.remove(id);
                    }
                    _ => {}
                }
            } else if let Some(signal) = signal.as_any().downcast_ref() {
                match signal {
                    RelativeLayoutListenerSignal::Register(relative_to) => {
                        if let (Some(item), Some(rect)) = (
                            app.layout_data().items.get(relative_to),
                            app.layout_data().rect_relative_to(id, relative_to),
                        ) {
                            self.relative_layout_listeners.insert(
                                id.to_owned(),
                                (relative_to.to_owned(), item.local_space.size(), rect),
                            );
                            to_relative_layout.insert(id.to_owned());
                        }
                    }
                    RelativeLayoutListenerSignal::Unregister => {
                        self.relative_layout_listeners.remove(id);
                    }
                    _ => {}
                }
            } else if let Some(signal) = signal.as_any().downcast_ref() {
                match signal {
                    NavSignal::Register(t) => match t {
                        NavType::Container => {
                            self.containers.insert(id.to_owned(), Default::default());
                        }
                        NavType::Item => {
                            if let Some((key, items)) = self
                                .containers
                                .iter_mut()
                                .filter(|(k, _)| {
                                    k.path() != id.path() && id.path().starts_with(k.path())
                                })
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
                        NavType::ScrollView => {
                            self.scroll_views.insert(id.to_owned());
                        }
                        NavType::ScrollViewContent => {
                            self.scroll_view_contents.insert(id.to_owned());
                        }
                        NavType::Tracking(who) => {
                            if let Some(who) = who.read() {
                                self.tracking.insert(id.to_owned(), who);
                            }
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
                            if let Some(key) = self.items_owners.remove(id)
                                && let Some(items) = self.containers.get_mut(&key)
                            {
                                items.remove(&key);
                            }
                            if let Some(lid) = &self.locked_widget
                                && lid == id
                            {
                                self.locked_widget = None;
                            }
                        }
                        NavType::Button => {
                            self.buttons.remove(id);
                        }
                        NavType::TextInput => {
                            self.text_inputs.remove(id);
                            if let Some(focused) = &self.focused_text_input
                                && focused == id
                            {
                                self.focused_text_input = None;
                            }
                        }
                        NavType::ScrollView => {
                            self.scroll_views.remove(id);
                        }
                        NavType::ScrollViewContent => {
                            self.scroll_view_contents.remove(id);
                        }
                        NavType::Tracking(_) => {
                            self.tracking.remove(id);
                        }
                    },
                    NavSignal::Select(idref) => to_select = Some(idref.to_owned()),
                    NavSignal::Unselect => to_select = Some(().into()),
                    NavSignal::Lock => {
                        if self.locked_widget.is_none() {
                            self.locked_widget = Some(id.to_owned());
                        }
                    }
                    NavSignal::Unlock => {
                        if let Some(lid) = &self.locked_widget
                            && lid == id
                        {
                            self.locked_widget = None;
                        }
                    }
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

        for (k, v) in &mut self.resize_listeners {
            if let Some(item) = app.layout_data().items.get(k) {
                let size = item.local_space.size();
                if to_resize.contains(k)
                    || (v.x - size.x).abs() >= 1.0e-6
                    || (v.y - size.y).abs() >= 1.0e-6
                {
                    app.send_message(k, ResizeListenerSignal::Change(size));
                    *v = size;
                }
            }
        }
        for (k, (r, s, v)) in &mut self.relative_layout_listeners {
            if let (Some(item), Some(rect)) = (
                app.layout_data().items.get(r),
                app.layout_data().rect_relative_to(k, r),
            ) {
                let size = item.local_space.size();
                if to_relative_layout.contains(k)
                    || (s.x - size.x).abs() >= 1.0e-6
                    || (s.y - size.y).abs() >= 1.0e-6
                    || (v.left - rect.left).abs() >= 1.0e-6
                    || (v.right - rect.right).abs() >= 1.0e-6
                    || (v.top - rect.top).abs() >= 1.0e-6
                    || (v.bottom - rect.bottom).abs() >= 1.0e-6
                {
                    app.send_message(k, RelativeLayoutListenerSignal::Change(size, rect));
                    *s = size;
                    *v = rect;
                }
            }
        }
        if !to_jump.is_empty() {
            self.cache_sorted_items_ids(app);
        }
        if let Some(idref) = to_select {
            self.select_item(app, idref.read());
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
                    NavSignal::Select(idref) => {
                        self.select_item(app, idref.read());
                    }
                    NavSignal::Unselect => {
                        self.select_item(app, None);
                    }
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
                    NavSignal::Jump(jump) => match jump {
                        NavJump::Scroll(NavScroll::Factor(_, _))
                        | NavJump::Scroll(NavScroll::Units(_, _))
                        | NavJump::Scroll(NavScroll::Widget(_, _)) => {
                            if let Some(id) = self.selected_scroll_view().cloned() {
                                self.jump(app, &id, jump);
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                },
                Interaction::PointerMove(Vec2 { x, y }) => {
                    if self.locked_widget.is_some() {
                        if self.selected_button().is_some() {
                            result.captured_pointer_location = true;
                        }
                    } else if let Some((found, _)) = self.find_button(app, x, y) {
                        result.captured_pointer_location = true;
                        self.select_item(app, Some(found));
                    } else {
                        if self.deselect_when_no_button_found {
                            self.select_item(app, None);
                        }
                        if self.does_hover_widget(app, x, y) {
                            result.captured_pointer_location = true;
                        }
                    }
                    for (id, who) in &self.tracking {
                        if let Some(layout) = app.layout_data().items.get(who) {
                            let rect = layout.ui_space;
                            let size = rect.size();
                            app.send_message(
                                id,
                                NavSignal::Axis(
                                    "pointer-x".to_owned(),
                                    if size.x > 0.0 {
                                        (x - rect.left) / size.x
                                    } else {
                                        0.0
                                    },
                                ),
                            );
                            app.send_message(
                                id,
                                NavSignal::Axis(
                                    "pointer-y".to_owned(),
                                    if size.y > 0.0 {
                                        (y - rect.top) / size.y
                                    } else {
                                        0.0
                                    },
                                ),
                            );
                            app.send_message(
                                id,
                                NavSignal::Axis("pointer-x-unscaled".to_owned(), x - rect.left),
                            );
                            app.send_message(
                                id,
                                NavSignal::Axis("pointer-y-unscaled".to_owned(), y - rect.top),
                            );
                            app.send_message(id, NavSignal::Axis("pointer-x-ui".to_owned(), x));
                            app.send_message(id, NavSignal::Axis("pointer-y-ui".to_owned(), y));
                            result.captured_pointer_location = true;
                            result.captured_pointer_action = true;
                        }
                    }
                }
                Interaction::PointerDown(button, Vec2 { x, y }) => {
                    if let Some((found, _)) = self.find_button(app, x, y) {
                        self.select_item(app, Some(found));
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
                    for (id, who) in &self.tracking {
                        if let Some(layout) = app.layout_data().items.get(who) {
                            let rect = layout.ui_space;
                            let size = rect.size();
                            app.send_message(
                                id,
                                NavSignal::Axis(
                                    "pointer-x".to_owned(),
                                    if size.x > 0.0 {
                                        (x - rect.left) / size.x
                                    } else {
                                        0.0
                                    },
                                ),
                            );
                            app.send_message(
                                id,
                                NavSignal::Axis(
                                    "pointer-y".to_owned(),
                                    if size.y > 0.0 {
                                        (y - rect.top) / size.y
                                    } else {
                                        0.0
                                    },
                                ),
                            );
                            app.send_message(
                                id,
                                NavSignal::Axis("pointer-x-unscaled".to_owned(), x - rect.left),
                            );
                            app.send_message(
                                id,
                                NavSignal::Axis("pointer-y-unscaled".to_owned(), y - rect.top),
                            );
                            app.send_message(id, NavSignal::Axis("pointer-x-ui".to_owned(), x));
                            app.send_message(id, NavSignal::Axis("pointer-y-ui".to_owned(), y));
                            result.captured_pointer_location = true;
                            result.captured_pointer_action = true;
                        }
                    }
                }
                Interaction::PointerUp(button, Vec2 { x, y }) => {
                    let action = match button {
                        PointerButton::Trigger => NavSignal::Accept(false),
                        PointerButton::Context => NavSignal::Context(false),
                    };
                    if self.send_to_selected_button(app, action) {
                        result.captured_pointer_action = true;
                    }
                    for (id, who) in &self.tracking {
                        if let Some(layout) = app.layout_data().items.get(who) {
                            let rect = layout.ui_space;
                            let size = rect.size();
                            app.send_message(
                                id,
                                NavSignal::Axis(
                                    "pointer-x".to_owned(),
                                    if size.x > 0.0 {
                                        (x - rect.left) / size.x
                                    } else {
                                        0.0
                                    },
                                ),
                            );
                            app.send_message(
                                id,
                                NavSignal::Axis(
                                    "pointer-y".to_owned(),
                                    if size.y > 0.0 {
                                        (y - rect.top) / size.y
                                    } else {
                                        0.0
                                    },
                                ),
                            );
                            app.send_message(
                                id,
                                NavSignal::Axis("pointer-x-unscaled".to_owned(), x - rect.left),
                            );
                            app.send_message(
                                id,
                                NavSignal::Axis("pointer-y-unscaled".to_owned(), y - rect.top),
                            );
                            app.send_message(id, NavSignal::Axis("pointer-x-ui".to_owned(), x));
                            app.send_message(id, NavSignal::Axis("pointer-y-ui".to_owned(), y));
                            result.captured_pointer_location = true;
                            result.captured_pointer_action = true;
                        }
                    }
                }
            }
        }
        Ok(result)
    }
}
