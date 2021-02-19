use crate::{
    unpack_named_slots, widget,
    widget::{component::containers::size_box::SizeBoxProps, unit::size::SizeBoxNode, WidgetId},
    widget_component, widget_hook,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonMessage {
    #[serde(default)]
    pub sender: WidgetId,
    #[serde(default)]
    pub action: ButtonAction,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ButtonSettingsProps {
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notify: Option<WidgetId>,
}
implement_props_data!(ButtonSettingsProps);

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ButtonProps {
    #[serde(default)]
    pub selected: bool,
    #[serde(default)]
    pub trigger: bool,
    #[serde(default)]
    pub context: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub text: Vec<TextChange>,
}
implement_props_data!(ButtonProps);

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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    use_button(life_cycle) {
        life_cycle.mount(|context| {
            drop(context.state.write(ButtonProps::default()));
            context.signals.write(ButtonSignal::Register);
        });

        life_cycle.unmount(|context| {
            context.signals.write(ButtonSignal::Unregister);
        });

        life_cycle.change(|context| {
            let ButtonSettingsProps { disabled, notify } = context.props.read_cloned_or_default();
            let mut data = match context.state.read::<ButtonProps>() {
                Ok(state) => state.clone(),
                Err(_) => ButtonProps::default(),
            };
            let empty = data.text.is_empty();
            data.text.clear();
            let mut dirty = false;
            for msg in context.messenger.messages {
                if let Some(action) = msg.downcast_ref::<ButtonAction>() {
                    match action {
                        ButtonAction::Select => {
                            if !disabled {
                                data.selected = true;
                                dirty = true;
                            }
                        }
                        ButtonAction::Unselect => {
                            data.selected = false;
                            dirty = true;
                        }
                        ButtonAction::TriggerStart => {
                            if !disabled {
                                data.trigger = true;
                                dirty = true;
                            }
                        }
                        ButtonAction::TriggerStop => {
                            data.trigger = false;
                            context.signals.write(ButtonSignal::Trigger);
                            dirty = true;
                        }
                        ButtonAction::TriggerCancel => {
                            data.trigger = false;
                            dirty = true;
                        }
                        ButtonAction::ContextStart => {
                            if !disabled {
                                data.context = true;
                                dirty = true;
                            }
                        }
                        ButtonAction::ContextStop => {
                            data.context = false;
                            context.signals.write(ButtonSignal::Context);
                            dirty = true;
                        }
                        ButtonAction::ContextCancel => {
                            data.context = false;
                            dirty = true;
                        }
                        ButtonAction::TextChange(change) => {
                            if !disabled {
                                data.text.push(*change);
                                dirty = true;
                            }
                        }
                        _ => {}
                    }
                    if let Some(ref notify) = notify {
                        context.messenger.write(notify.to_owned(), ButtonMessage {
                            sender: context.id.to_owned(),
                            action: *action,
                        });
                    }
                }
            }
            if dirty || data.text.is_empty() != empty {
                drop(context.state.write(data));
            }
        });
    }
}

widget_component! {
    pub button(id, props, state, named_slots) [use_button] {
        unpack_named_slots!(named_slots => content);
        let SizeBoxProps { width, height, margin, transform } = props.read_cloned_or_default();
        if let Some(props) = content.props_mut() {
            let s = match state.read::<ButtonProps>() {
                Ok(state) => state.clone(),
                Err(_) => ButtonProps::default(),
            };
            props.write(s)
        }

        widget! {{{
            SizeBoxNode {
                id: id.to_owned(),
                props: props.clone(),
                slot: Box::new(content),
                width,
                height,
                margin,
                transform,
            }
        }}}
    }
}
