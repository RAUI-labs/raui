use crate::{
    messenger::MessageData,
    pre_hooks, unpack_named_slots, widget,
    widget::{
        component::interactive::navigation::{use_nav_button, use_nav_item, NavSignal},
        context::{WidgetContext, WidgetMountOrChangeContext},
        node::WidgetNode,
        unit::area::AreaBoxNode,
        utils::Vec2,
        WidgetId, WidgetIdOrRef,
    },
    MessageData, PropsData,
};
use serde::{Deserialize, Serialize};

fn is_false(v: &bool) -> bool {
    !*v
}

fn is_zero(v: &Vec2) -> bool {
    v.x.abs() < 1.0e-6 && v.y.abs() < 1.0e-6
}

#[derive(PropsData, Debug, Default, Copy, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct ButtonProps {
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub selected: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub trigger: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub context: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero")]
    pub pointer: Vec2,
}

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct ButtonNotifyProps(
    #[serde(default)]
    #[serde(skip_serializing_if = "WidgetIdOrRef::is_none")]
    pub WidgetIdOrRef,
);

#[derive(MessageData, Debug, Clone)]
#[message_data(crate::messenger::MessageData)]
pub struct ButtonNotifyMessage {
    pub sender: WidgetId,
    pub state: ButtonProps,
    pub prev: ButtonProps,
}

impl ButtonNotifyMessage {
    pub fn select_start(&self) -> bool {
        !self.prev.selected && self.state.selected
    }

    pub fn select_stop(&self) -> bool {
        self.prev.selected && !self.state.selected
    }

    pub fn select_changed(&self) -> bool {
        self.prev.selected != self.state.selected
    }

    pub fn trigger_start(&self) -> bool {
        !self.prev.trigger && self.state.trigger
    }

    pub fn trigger_stop(&self) -> bool {
        self.prev.trigger && !self.state.trigger
    }

    pub fn trigger_changed(&self) -> bool {
        self.prev.trigger != self.state.trigger
    }

    pub fn context_start(&self) -> bool {
        !self.prev.context && self.state.context
    }

    pub fn context_stop(&self) -> bool {
        self.prev.context && !self.state.context
    }

    pub fn context_changed(&self) -> bool {
        self.prev.context != self.state.context
    }
}

pub fn use_button_notified_state(context: &mut WidgetContext) {
    context.life_cycle.change(|context| {
        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<ButtonNotifyMessage>() {
                let _ = context.state.write_with(msg.state);
            }
        }
    });
}

#[pre_hooks(use_nav_item, use_nav_button)]
pub fn use_button(context: &mut WidgetContext) {
    fn notify<T>(context: &WidgetMountOrChangeContext, data: T)
    where
        T: 'static + MessageData,
    {
        if let Ok(ButtonNotifyProps(notify)) = context.props.read() {
            if let Some(to) = notify.read() {
                context.messenger.write(to, data);
            }
        }
    }

    context.life_cycle.mount(|context| {
        notify(
            &context,
            ButtonNotifyMessage {
                sender: context.id.to_owned(),
                state: Default::default(),
                prev: Default::default(),
            },
        );
        let _ = context.state.write_with(ButtonProps::default());
    });

    context.life_cycle.change(|context| {
        let mut data = context.state.read_cloned_or_default::<ButtonProps>();
        let prev = data;
        let mut dirty = false;
        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref() {
                match msg {
                    NavSignal::Select(_) => {
                        data.selected = true;
                        dirty = true;
                    }
                    NavSignal::Unselect => {
                        data.selected = false;
                        dirty = true;
                    }
                    NavSignal::Accept(v) => {
                        data.trigger = *v;
                        dirty = true;
                    }
                    NavSignal::Context(v) => {
                        data.context = *v;
                        dirty = true;
                    }
                    NavSignal::Axis(n, v) => match n.as_str() {
                        "pointer-x" => {
                            data.pointer.x = *v;
                            dirty = true;
                        }
                        "pointer-y" => {
                            data.pointer.y = *v;
                            dirty = true;
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
        if dirty {
            notify(
                &context,
                ButtonNotifyMessage {
                    sender: context.id.to_owned(),
                    state: data.to_owned(),
                    prev,
                },
            );
            let _ = context.state.write_with(data);
        }
    });
}

#[pre_hooks(use_button)]
pub fn button(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        state,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => content);

    if let Some(p) = content.props_mut() {
        p.write(state.read_cloned_or_default::<ButtonProps>());
    }

    widget! {{{
        AreaBoxNode {
            id: id.to_owned(),
            slot: Box::new(content),
            renderer_effect: None,
        }
    }}}
}
