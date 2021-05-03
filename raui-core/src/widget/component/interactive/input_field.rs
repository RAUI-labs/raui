use crate::{
    messenger::MessageData,
    pre_hooks, unpack_named_slots, widget,
    widget::{
        component::interactive::{
            button::{use_button, ButtonProps},
            navigation::{use_nav_item, use_nav_text_input, NavSignal, NavTextChange},
        },
        context::{WidgetContext, WidgetMountOrChangeContext},
        node::WidgetNode,
        unit::area::AreaBoxNode,
        WidgetId, WidgetIdOrRef,
    },
    MessageData, PropsData,
};
use serde::{Deserialize, Serialize};

fn is_false(v: &bool) -> bool {
    !*v
}

fn is_zero(v: &usize) -> bool {
    *v == 0
}

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
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

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct TextInputNotifyProps(
    #[serde(default)]
    #[serde(skip_serializing_if = "WidgetIdOrRef::is_none")]
    pub WidgetIdOrRef,
);

#[derive(MessageData, Debug, Clone)]
#[message_data(crate::messenger::MessageData)]
pub struct TextInputNotifyMessage {
    pub sender: WidgetId,
    pub state: TextInputProps,
}

pub fn use_text_input_notified_state(context: &mut WidgetContext) {
    context.life_cycle.change(|context| {
        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<TextInputNotifyMessage>() {
                let _ = context.state.write_with(msg.state.to_owned());
            }
        }
    });
}

#[pre_hooks(use_nav_text_input)]
pub fn use_text_input(context: &mut WidgetContext) {
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

    context.life_cycle.mount(|context| {
        let mut data = context.props.read_cloned_or_default::<TextInputProps>();
        data.focused = false;
        notify(
            &context,
            TextInputNotifyMessage {
                sender: context.id.to_owned(),
                state: data.to_owned(),
            },
        );
        let _ = context.state.write_with(data);
    });

    context.life_cycle.change(|context| {
        let mut data = context.state.read_cloned_or_default::<TextInputProps>();
        let mut dirty = false;
        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<NavSignal>() {
                match msg {
                    NavSignal::FocusTextInput(idref) => {
                        data.focused = idref.is_some();
                        dirty = true;
                    }
                    NavSignal::TextChange(change) => {
                        if data.focused {
                            match change {
                                NavTextChange::InsertCharacter(c) => {
                                    if !c.is_control() {
                                        data.cursor_position =
                                            data.cursor_position.min(data.text.len());
                                        data.text.insert(data.cursor_position, *c);
                                        data.cursor_position += 1;
                                    }
                                }
                                NavTextChange::MoveCursorLeft => {
                                    if data.cursor_position > 0 {
                                        data.cursor_position -= 1;
                                    }
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
                                    if data.cursor_position > 0
                                        && data.cursor_position <= data.text.len()
                                    {
                                        data.cursor_position -= 1;
                                        data.text.remove(data.cursor_position);
                                    }
                                }
                                NavTextChange::DeleteRight => {
                                    if data.cursor_position < data.text.len() {
                                        data.text.remove(data.cursor_position);
                                    }
                                }
                                NavTextChange::NewLine => {
                                    if data.allow_new_line {
                                        data.cursor_position =
                                            data.cursor_position.min(data.text.len());
                                        data.text.insert(data.cursor_position, '\n');
                                        data.cursor_position += 1;
                                    }
                                }
                            }
                            data.cursor_position = data.cursor_position.min(data.text.len());
                            dirty = true;
                        }
                    }
                    _ => {}
                }
            }
        }
        if dirty {
            notify(
                &context,
                TextInputNotifyMessage {
                    sender: context.id.to_owned(),
                    state: data.to_owned(),
                },
            );
            let _ = context.state.write_with(data);
        }
    });
}

#[pre_hooks(use_button, use_text_input)]
pub fn use_input_field(context: &mut WidgetContext) {
    context.life_cycle.change(|context| {
        let focused = context
            .state
            .map_or_default::<TextInputProps, _, _>(|s| s.focused);
        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<NavSignal>() {
                match msg {
                    NavSignal::Accept(true) => {
                        if !focused {
                            context
                                .signals
                                .write(NavSignal::FocusTextInput(context.id.to_owned().into()));
                        }
                    }
                    NavSignal::Cancel(true) => {
                        if focused {
                            context.signals.write(NavSignal::FocusTextInput(().into()));
                        }
                    }
                    _ => {}
                }
            }
        }
    });
}

#[pre_hooks(use_nav_item, use_text_input)]
pub fn text_input(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        state,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => content);

    if let Some(p) = content.props_mut() {
        p.write(state.read_cloned_or_default::<TextInputProps>());
    }

    widget! {{{
        AreaBoxNode {
            id: id.to_owned(),
            slot: Box::new(content),
            renderer_effect: None,
        }
    }}}
}

#[pre_hooks(use_nav_item, use_input_field)]
pub fn input_field(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        state,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => content);

    if let Some(p) = content.props_mut() {
        p.write(state.read_cloned_or_default::<ButtonProps>());
        p.write(state.read_cloned_or_default::<TextInputProps>());
    }

    widget! {{{
        AreaBoxNode {
            id: id.to_owned(),
            slot: Box::new(content),
            renderer_effect: None,
        }
    }}}
}
