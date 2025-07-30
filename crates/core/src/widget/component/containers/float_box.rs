use crate::{
    MessageData, PropsData, Scalar, make_widget, pre_hooks,
    widget::{
        WidgetId, WidgetIdOrRef,
        component::{
            containers::content_box::{ContentBoxProps, content_box},
            interactive::navigation::{
                NavContainerActive, NavItemActive, NavJumpActive, use_nav_container_active,
                use_nav_item, use_nav_jump_direction_active,
            },
        },
        context::{WidgetContext, WidgetMountOrChangeContext},
        node::WidgetNode,
        unit::content::ContentBoxContentReposition,
        utils::Vec2,
    },
};
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Copy, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct FloatBoxProps {
    #[serde(default)]
    pub bounds_left: Option<Scalar>,
    #[serde(default)]
    pub bounds_right: Option<Scalar>,
    #[serde(default)]
    pub bounds_top: Option<Scalar>,
    #[serde(default)]
    pub bounds_bottom: Option<Scalar>,
}

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct FloatBoxNotifyProps(
    #[serde(default)]
    #[serde(skip_serializing_if = "WidgetIdOrRef::is_none")]
    pub WidgetIdOrRef,
);

#[derive(PropsData, Debug, Copy, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct FloatBoxState {
    #[serde(default)]
    pub position: Vec2,
    #[serde(default = "FloatBoxState::default_zoom")]
    pub zoom: Scalar,
}

impl Default for FloatBoxState {
    fn default() -> Self {
        Self {
            position: Default::default(),
            zoom: Self::default_zoom(),
        }
    }
}

impl FloatBoxState {
    fn default_zoom() -> Scalar {
        1.0
    }
}

#[derive(MessageData, Debug, Default, Clone)]
#[message_data(crate::messenger::MessageData)]
pub struct FloatBoxNotifyMessage {
    pub sender: WidgetId,
    pub state: FloatBoxState,
    pub prev: FloatBoxState,
}

#[derive(MessageData, Debug, Clone)]
#[message_data(crate::messenger::MessageData)]
pub struct FloatBoxChangeMessage {
    pub sender: WidgetId,
    pub change: FloatBoxChange,
}

#[derive(Debug, Clone)]
pub enum FloatBoxChange {
    Absolute(FloatBoxState),
    RelativePosition(Vec2),
    RelativeZoom(Scalar),
}

pub fn use_float_box(context: &mut WidgetContext) {
    fn notify(context: &WidgetMountOrChangeContext, data: FloatBoxNotifyMessage) {
        if let Ok(FloatBoxNotifyProps(notify)) = context.props.read() {
            if let Some(to) = notify.read() {
                context.messenger.write(to, data);
            }
        }
    }

    context.life_cycle.mount(|context| {
        let props = context.props.read_cloned_or_default::<FloatBoxProps>();
        let mut data = context.props.read_cloned_or_default::<FloatBoxState>();
        if let Some(limit) = props.bounds_left {
            data.position.x = data.position.x.max(limit);
        }
        if let Some(limit) = props.bounds_right {
            data.position.x = data.position.x.min(limit);
        }
        if let Some(limit) = props.bounds_top {
            data.position.y = data.position.y.max(limit);
        }
        if let Some(limit) = props.bounds_bottom {
            data.position.y = data.position.y.min(limit);
        }
        notify(
            &context,
            FloatBoxNotifyMessage {
                sender: context.id.to_owned(),
                state: data,
                prev: data,
            },
        );
        let _ = context.state.write_with(data);
    });

    context.life_cycle.change(|context| {
        let props = context.props.read_cloned_or_default::<FloatBoxProps>();
        let mut dirty = false;
        let mut data = context.state.read_cloned_or_default::<FloatBoxState>();
        let prev = data;
        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<FloatBoxChangeMessage>() {
                match msg.change {
                    FloatBoxChange::Absolute(value) => {
                        data = value;
                        dirty = true;
                    }
                    FloatBoxChange::RelativePosition(delta) => {
                        data.position.x -= delta.x;
                        data.position.y -= delta.y;
                        dirty = true;
                    }
                    FloatBoxChange::RelativeZoom(delta) => {
                        data.zoom *= delta;
                        dirty = true;
                    }
                }
            }
        }
        if dirty {
            if let Some(limit) = props.bounds_left {
                data.position.x = data.position.x.max(limit);
            }
            if let Some(limit) = props.bounds_right {
                data.position.x = data.position.x.min(limit);
            }
            if let Some(limit) = props.bounds_top {
                data.position.y = data.position.y.max(limit);
            }
            if let Some(limit) = props.bounds_bottom {
                data.position.y = data.position.y.min(limit);
            }
            notify(
                &context,
                FloatBoxNotifyMessage {
                    sender: context.id.to_owned(),
                    state: data.to_owned(),
                    prev,
                },
            );
            let _ = context.state.write_with(data);
        }
    });
}

#[pre_hooks(use_nav_container_active, use_nav_jump_direction_active, use_nav_item)]
pub fn nav_float_box(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        props,
        listed_slots,
        ..
    } = context;

    let props = props
        .clone()
        .without::<NavContainerActive>()
        .without::<NavJumpActive>()
        .without::<NavItemActive>();

    make_widget!(float_box)
        .key(key)
        .merge_props(props)
        .listed_slots(listed_slots)
        .into()
}

#[pre_hooks(use_float_box)]
pub fn float_box(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        props,
        state,
        mut listed_slots,
        ..
    } = context;

    let mut props = props.read_cloned_or_default::<ContentBoxProps>();
    let state = state.read_cloned_or_default::<FloatBoxState>();
    props.content_reposition = ContentBoxContentReposition {
        offset: Vec2 {
            x: -state.position.x,
            y: -state.position.y,
        },
        scale: Vec2 {
            x: state.zoom,
            y: state.zoom,
        },
    };

    for item in listed_slots.iter_mut() {
        if let Some(p) = item.props_mut() {
            p.write(state);
            if !p.has::<FloatBoxNotifyProps>() {
                p.write(FloatBoxNotifyProps(context.id.to_owned().into()));
            }
        }
    }

    make_widget!(content_box)
        .key(key)
        .with_props(props)
        .listed_slots(listed_slots)
        .into()
}
