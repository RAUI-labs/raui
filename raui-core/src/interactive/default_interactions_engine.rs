use crate::{
    application::Application,
    interactive::InteractionsEngine,
    widget::{
        component::interactive::button::{ButtonAction, ButtonSignal, TextChange},
        unit::WidgetUnit,
        WidgetId,
    },
    Scalar,
};
use std::collections::{HashSet, VecDeque};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PointerButton {
    Trigger,
    Context,
}

#[derive(Debug, Clone)]
pub enum Interaction {
    None,
    Select(WidgetId),
    Unselect,
    PointerMove(Scalar, Scalar),
    PointerDown(PointerButton, Scalar, Scalar),
    PointerUp(PointerButton, Scalar, Scalar),
    // AxisChange(usize, Scalar, Scalar),
    TextChange(TextChange),
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
    buttons: HashSet<WidgetId>,
    interactions_queue: VecDeque<Interaction>,
    selected: Option<WidgetId>,
}

impl DefaultInteractionsEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(buttons: usize, interactions_queue: usize) -> Self {
        Self {
            buttons: HashSet::with_capacity(buttons),
            interactions_queue: VecDeque::with_capacity(interactions_queue),
            selected: None,
        }
    }

    pub fn interact(&mut self, interaction: Interaction) {
        if interaction.is_some() {
            self.interactions_queue.push_back(interaction);
        }
    }

    pub fn clear_queue(&mut self, put_unselect: bool) {
        self.interactions_queue.clear();
        if put_unselect {
            self.interactions_queue.push_back(Interaction::Unselect);
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

    fn select_button(&mut self, app: &mut Application, id: Option<WidgetId>) {
        if self.selected != id {
            if let Some(selected) = self.selected.as_ref() {
                app.send_message(selected, ButtonAction::Unselect);
            }
            self.selected = id;
            if let Some(selected) = self.selected.as_ref() {
                app.send_message(selected, ButtonAction::Select);
            }
        }
    }
}

impl InteractionsEngine<DefaultInteractionsEngineResult, ()> for DefaultInteractionsEngine {
    fn perform_interactions(
        &mut self,
        app: &mut Application,
    ) -> Result<DefaultInteractionsEngineResult, ()> {
        for (id, signal) in app.signals() {
            if let Some(signal) = signal.downcast_ref::<ButtonSignal>() {
                match signal {
                    ButtonSignal::Register => {
                        self.buttons.insert(id.to_owned());
                    }
                    ButtonSignal::Unregister => {
                        self.buttons.remove(id);
                    }
                    _ => {}
                }
            }
        }
        let mut result = DefaultInteractionsEngineResult::default();
        while let Some(interaction) = self.interactions_queue.pop_front() {
            match interaction {
                Interaction::None => {}
                Interaction::Select(id) => {
                    self.select_button(app, Some(id));
                }
                Interaction::Unselect => {
                    self.select_button(app, None);
                }
                Interaction::PointerMove(x, y) => {
                    let found = self.find_button(app, x, y);
                    if found.is_some() {
                        self.select_button(app, found);
                        result.captured_pointer_location = true;
                    } else {
                        self.select_button(app, None);
                        if self.does_hover_widget(app, x, y) {
                            result.captured_pointer_location = true;
                        }
                    }
                }
                Interaction::PointerDown(button, _, _) => {
                    if let Some(id) = &self.selected {
                        match button {
                            PointerButton::Trigger => {
                                app.send_message(id, ButtonAction::TriggerStart);
                            }
                            PointerButton::Context => {
                                app.send_message(id, ButtonAction::ContextStart);
                            }
                        }
                        result.captured_pointer_action = true;
                    }
                }
                Interaction::PointerUp(button, _, _) => {
                    if let Some(id) = &self.selected {
                        match button {
                            PointerButton::Trigger => {
                                app.send_message(id, ButtonAction::TriggerStop);
                            }
                            PointerButton::Context => {
                                app.send_message(id, ButtonAction::ContextStop);
                            }
                        }
                        result.captured_pointer_action = true;
                    }
                }
                // Interaction::AxisChange(axis, x, y) => {}
                Interaction::TextChange(change) => {
                    if let Some(id) = &self.selected {
                        app.send_message(id, ButtonAction::TextChange(change));
                        result.captured_text_change = true;
                    }
                }
            }
        }
        Ok(result)
    }
}
