use crate::{
    messenger::MessageData,
    unpack_named_slots, widget,
    widget::{
        component::interactive::navigation::{use_nav_button, use_nav_item, NavSignal},
        context::WidgetMountOrChangeContext,
        unit::area::AreaBoxNode,
        utils::Vec2,
        WidgetId, WidgetIdOrRef,
    },
    widget_component, widget_hook,
};
use serde::{Deserialize, Serialize};

fn is_false(v: &bool) -> bool {
    !*v
}

fn is_zero(v: &Vec2) -> bool {
    v.x.abs() < 1.0e-6 && v.y.abs() < 1.0e-6
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
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
implement_props_data!(ButtonProps);

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ButtonNotifyProps(
    #[serde(default)]
    #[serde(skip_serializing_if = "WidgetIdOrRef::is_none")]
    pub WidgetIdOrRef,
);
implement_props_data!(ButtonNotifyProps);

#[derive(Debug, Clone)]
pub struct ButtonNotifyMessage {
    pub sender: WidgetId,
    pub state: ButtonProps,
    pub prev: ButtonProps,
}
implement_message_data!(ButtonNotifyMessage);

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

widget_hook! {
    pub use_button_notified_state(life_cycle) {
        life_cycle.change(|context| {
            for msg in context.messenger.messages {
                if let Some(msg) = msg.as_any().downcast_ref::<ButtonNotifyMessage>() {
                    drop(context.state.write_with(msg.state));
                }
            }
        });
    }
}

widget_hook! {
    pub use_button(life_cycle) [use_nav_button] {
        fn notify<T>(context: &WidgetMountOrChangeContext, data: T)
        where
            T: 'static + MessageData,
        {
            if let Ok(notify) = context.props.read::<ButtonNotifyProps>() {
                if let Some(to) = notify.0.read() {
                    context.messenger.write(to, data);
                }
            }
        }

        life_cycle.mount(|context| {
            notify(&context, ButtonNotifyMessage {
                sender: context.id.to_owned(),
                state: ButtonProps::default(),
                prev: ButtonProps::default(),
            });
            drop(context.state.write_with(ButtonProps::default()));
        });

        life_cycle.change(|context| {
            let mut data = context.state.read_cloned_or_default::<ButtonProps>();
            let prev = data;
            let mut dirty = false;
            for msg in context.messenger.messages {
                if let Some(msg) = msg.as_any().downcast_ref::<NavSignal>() {
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
                        }
                        _ => {}
                    }
                }
            }
            if dirty {
                notify(&context, ButtonNotifyMessage {
                    sender: context.id.to_owned(),
                    state: data.to_owned(),
                    prev,
                });
                drop(context.state.write_with(data));
            }
        });
    }
}

widget_component! {
    pub button(id, state, named_slots) [use_nav_item, use_button] {
        unpack_named_slots!(named_slots => content);

        if let Some(p) = content.props_mut() {
            p.write(state.read_cloned_or_default::<ButtonProps>());
        }

        widget! {{{
            AreaBoxNode {
                id: id.to_owned(),
                slot: Box::new(content),
            }
        }}}
    }
}
