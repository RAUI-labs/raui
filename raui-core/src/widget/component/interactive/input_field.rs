use crate::{
    messenger::MessageData,
    widget,
    widget::{
        component::interactive::{
            button::{use_button, ButtonProps},
            navigation::{use_nav_item, use_nav_text_input, NavSignal, NavTextChange},
        },
        context::WidgetMountOrChangeContext,
        unit::area::AreaBoxNode,
        WidgetId, WidgetIdOrRef,
    },
    widget_component, widget_hook,
};
use serde::{Deserialize, Serialize};

fn is_false(v: &bool) -> bool {
    !*v
}

fn is_zero(v: &usize) -> bool {
    *v == 0
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TextInputProps {
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub focused: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero")]
    pub cursor_position: usize,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub allow_new_line: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub text: String,
}
implement_props_data!(TextInputProps);

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TextInputNotifyProps(
    #[serde(default)]
    #[serde(skip_serializing_if = "WidgetIdOrRef::is_none")]
    pub WidgetIdOrRef,
);
implement_props_data!(TextInputNotifyProps);

#[derive(Debug, Clone)]
pub struct TextInputNotifyMessage {
    pub sender: WidgetId,
    pub state: TextInputProps,
}
implement_message_data!(TextInputNotifyMessage);

widget_hook! {
    pub use_text_input_notified_state(life_cycle) {
        life_cycle.change(|context| {
            for msg in context.messenger.messages {
                if let Some(msg) = msg.as_any().downcast_ref::<TextInputNotifyMessage>() {
                    drop(context.state.write_with(msg.state.to_owned()));
                }
            }
        });
    }
}

widget_hook! {
    pub use_text_input(life_cycle) [use_nav_text_input] {
        fn notify<T>(context: &WidgetMountOrChangeContext, data: T)
        where
            T: 'static + MessageData,
        {
            if let Ok(notify) = context.props.read::<TextInputNotifyProps>() {
                if let Some(to) = notify.0.read() {
                    context.messenger.write(to, data);
                }
            }
        }

        life_cycle.mount(|context| {
            let mut data = context.props.read_cloned_or_default::<TextInputProps>();
            data.focused = false;
            notify(&context, TextInputNotifyMessage {
                sender: context.id.to_owned(),
                state: data.to_owned(),
            });
            drop(context.state.write_with(data));
        });

        life_cycle.change(|context| {
            let mut data = context.state.read_cloned_or_default::<TextInputProps>();
            let mut dirty = false;
            for msg in context.messenger.messages {
                if let Some(msg) = msg.as_any().downcast_ref::<NavSignal>() {
                    match msg {
                        NavSignal::FocusTextInput(idref) => {
                            data.focused = idref.is_some();
                            dirty = true;
                        }
                        NavSignal::TextChange(change) => if data.focused {
                            match change {
                                NavTextChange::InsertCharacter(c) => if !c.is_control() {
                                    data.cursor_position = data.cursor_position.min(data.text.len());
                                    data.text.insert(data.cursor_position, *c);
                                    data.cursor_position += 1;
                                }
                                NavTextChange::MoveCursorLeft => if data.cursor_position > 0 {
                                    data.cursor_position -= 1;
                                }
                                NavTextChange::MoveCursorRight => {
                                    if data.cursor_position < data.text.len() {
                                        data.cursor_position += 1;
                                    }
                                }
                                NavTextChange::MoveCursorStart => data.cursor_position = 0,
                                NavTextChange::MoveCursorEnd => {
                                    data.cursor_position = data.text.len();
                                }
                                NavTextChange::DeleteLeft => {
                                    if data.cursor_position > 0 && data.cursor_position <= data.text.len() {
                                        data.cursor_position -= 1;
                                        data.text.remove(data.cursor_position);
                                    }
                                }
                                NavTextChange::DeleteRight => {
                                    if data.cursor_position < data.text.len() {
                                        data.text.remove(data.cursor_position);
                                    }
                                }
                                NavTextChange::NewLine => if data.allow_new_line {
                                    data.cursor_position = data.cursor_position.min(data.text.len());
                                    data.text.insert(data.cursor_position, '\n');
                                    data.cursor_position += 1;
                                }
                            }
                            data.cursor_position = data.cursor_position.min(data.text.len());
                            dirty = true;
                        }
                        _ => {}
                    }
                }
            }
            if dirty {
                notify(&context, TextInputNotifyMessage {
                    sender: context.id.to_owned(),
                    state: data.to_owned(),
                });
                drop(context.state.write_with(data));
            }
        });
    }
}

widget_hook! {
    pub use_input_field(life_cycle) [use_button, use_text_input] {
        life_cycle.change(|context| {
            let focused = context.state.map_or_default::<TextInputProps, _, _>(|s| s.focused);
            for msg in context.messenger.messages {
                if let Some(msg) = msg.as_any().downcast_ref::<NavSignal>() {
                    match msg {
                        NavSignal::Accept(true) => if !focused {
                            context.signals.write(NavSignal::FocusTextInput(
                                context.id.to_owned().into()
                            ));
                        }
                        NavSignal::Cancel(true) => if focused {
                            context.signals.write(NavSignal::FocusTextInput(().into()));
                        }
                        _ => {}
                    }
                }
            }
        });
    }
}

widget_component!(
    #[pre(use_nav_item, use_text_input)]
    pub fn text_input(id: Id, state: State, (content,): NamedSlots) {
        if let Some(p) = content.props_mut() {
            p.write(state.read_cloned_or_default::<TextInputProps>());
        }

        widget! {{{
            AreaBoxNode {
                id: id.to_owned(),
                slot: Box::new(content),
                ..Default::default()
            }
        }}}
    }
);

widget_component!(
    #[pre(use_nav_item, use_input_field)]
    pub fn input_field(id: Id, state: State, (content,): NamedSlots) {
        if let Some(p) = content.props_mut() {
            p.write(state.read_cloned_or_default::<ButtonProps>());
            p.write(state.read_cloned_or_default::<TextInputProps>());
        }

        widget! {{{
            AreaBoxNode {
                id: id.to_owned(),
                slot: Box::new(content),
                ..Default::default()
            }
        }}}
    }
);
