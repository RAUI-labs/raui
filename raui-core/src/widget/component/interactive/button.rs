use crate::{
    unpack_named_slots, widget,
    widget::{component::containers::size_box::SizeBoxProps, unit::size::SizeBoxNode},
    widget_component, widget_hook,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ButtonProps {
    pub selected: bool,
    pub trigger: bool,
    pub context: bool,
    pub text: Vec<TextChange>,
}
implement_props_data!(ButtonProps, "ButtonProps");

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextChange {
    InsertCharacter(char),
    MoveCursorLeft,
    MoveCursorRight,
    MoveCursorStart,
    MoveCursorEnd,
    DeleteLeft,
    DeleteRight,
    NewLine,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ButtonAction {
    None,
    Select,
    Unselect,
    TriggerStart,
    TriggerStop,
    TriggerCancel,
    ContextStart,
    ContextStop,
    ContextCancel,
    TextChange(TextChange),
}

impl Default for ButtonAction {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ButtonSignal {
    None,
    Register,
    Unregister,
    Trigger,
    Context,
}

impl Default for ButtonSignal {
    fn default() -> Self {
        Self::None
    }
}

widget_hook! {
    use_button life_cycle {
        life_cycle.mount(|id, _, state, _, signals| {
            drop(state.write(ButtonProps::default()));
            drop(signals.write(ButtonSignal::Register));
        });

        life_cycle.unmount(|id, _, _, signals| {
            drop(signals.write(ButtonSignal::Unregister));
        });

        life_cycle.change(|_, _, state, messenger, signals| {
            let mut data = match state.read::<ButtonProps>() {
                Ok(state) => state.clone(),
                Err(_) => ButtonProps::default(),
            };
            let empty = data.text.is_empty();
            data.text.clear();
            let mut dirty = false;
            for msg in messenger.messages {
                if let Some(action) = msg.downcast_ref::<ButtonAction>() {
                    match action {
                        ButtonAction::Select => {
                            data.selected = true;
                            dirty = true;
                        }
                        ButtonAction::Unselect => {
                            data.selected = false;
                            dirty = true;
                        }
                        ButtonAction::TriggerStart => {
                            data.trigger = true;
                            dirty = true;
                        }
                        ButtonAction::TriggerStop => {
                            data.trigger = false;
                            signals.write(ButtonSignal::Trigger);
                            dirty = true;
                        }
                        ButtonAction::TriggerCancel => {
                            data.trigger = false;
                            dirty = true;
                        }
                        ButtonAction::ContextStart => {
                            data.context = true;
                            dirty = true;
                        }
                        ButtonAction::ContextStop => {
                            data.context = false;
                            signals.write(ButtonSignal::Context);
                            dirty = true;
                        }
                        ButtonAction::ContextCancel => {
                            data.context = false;
                            dirty = true;
                        }
                        ButtonAction::TextChange(change) => {
                            data.text.push(*change);
                            dirty = true;
                        }
                        _ => {}
                    }
                }
            }
            if dirty || data.text.is_empty() != empty {
                drop(state.write(data));
            }
        });
    }
}

widget_component! {
    pub button(id, props, state, named_slots) [use_button] {
        unpack_named_slots!(named_slots => content);
        if let Some(props) = content.props_mut() {
            let s = match state.read::<ButtonProps>() {
                Ok(state) => state.clone(),
                Err(_) => ButtonProps::default(),
            };
            props.write(s)
        }
        let SizeBoxProps { width, height, margin } = props.read_cloned_or_default();

        widget! {{{
            SizeBoxNode {
                id: id.to_owned(),
                props: props.clone(),
                slot: Box::new(content),
                width,
                height,
                margin,
            }
        }}}
    }
}
