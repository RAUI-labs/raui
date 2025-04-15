use crate::{
    MessageData, PropsData,
    messenger::MessageData,
    pre_hooks,
    widget::{
        WidgetId, WidgetIdOrRef,
        component::interactive::navigation::{NavJump, NavScroll, NavSignal, use_nav_scroll_view},
        context::{WidgetContext, WidgetMountOrChangeContext},
        utils::Vec2,
    },
};
use serde::{Deserialize, Serialize};

fn is_zero(v: &Vec2) -> bool {
    v.x.abs() < 1.0e-6 && v.y.abs() < 1.0e-6
}

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct ScrollViewState {
    #[serde(default)]
    pub value: Vec2,
    #[serde(default)]
    pub size_factor: Vec2,
}

#[derive(PropsData, Debug, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct ScrollViewRange {
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero")]
    pub from: Vec2,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero")]
    pub to: Vec2,
}

impl Default for ScrollViewRange {
    fn default() -> Self {
        Self {
            from: Vec2 { x: 0.0, y: 0.0 },
            to: Vec2 { x: 1.0, y: 1.0 },
        }
    }
}

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct ScrollViewNotifyProps(
    #[serde(default)]
    #[serde(skip_serializing_if = "WidgetIdOrRef::is_none")]
    pub WidgetIdOrRef,
);

#[derive(MessageData, Debug, Clone)]
#[message_data(crate::messenger::MessageData)]
pub struct ScrollViewNotifyMessage {
    pub sender: WidgetId,
    pub state: ScrollViewState,
}

pub fn use_scroll_view_notified_state(context: &mut WidgetContext) {
    context.life_cycle.change(|context| {
        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<ScrollViewNotifyMessage>() {
                let _ = context.state.write_with(msg.state.clone());
            }
        }
    });
}

#[pre_hooks(use_nav_scroll_view)]
pub fn use_scroll_view(context: &mut WidgetContext) {
    fn notify<T>(context: &WidgetMountOrChangeContext, data: T)
    where
        T: 'static + MessageData,
    {
        if let Ok(notify) = context.props.read::<ScrollViewNotifyProps>() {
            if let Some(to) = notify.0.read() {
                context.messenger.write(to, data);
            }
        }
    }

    context.life_cycle.mount(|context| {
        notify(
            &context,
            ScrollViewNotifyMessage {
                sender: context.id.to_owned(),
                state: ScrollViewState::default(),
            },
        );
        let _ = context.state.write_with(ScrollViewState::default());
    });

    context.life_cycle.change(|context| {
        let mut dirty = false;
        let mut data = context.state.read_cloned_or_default::<ScrollViewState>();
        let range = context.props.read::<ScrollViewRange>();
        for msg in context.messenger.messages {
            if let Some(NavSignal::Jump(NavJump::Scroll(NavScroll::Change(
                value,
                factor,
                relative,
            )))) = msg.as_any().downcast_ref()
            {
                if *relative {
                    data.value.x += value.x;
                    data.value.y += value.y;
                } else {
                    data.value = *value;
                }
                if factor.x <= 1.0 {
                    data.value.x = 0.0;
                }
                if factor.y <= 1.0 {
                    data.value.y = 0.0;
                }
                if let Ok(range) = &range {
                    data.value.x = data.value.x.max(range.from.x).min(range.to.x);
                    data.value.y = data.value.y.max(range.from.y).min(range.to.y);
                }
                data.size_factor = *factor;
                dirty = true;
            }
        }
        if dirty {
            notify(
                &context,
                ScrollViewNotifyMessage {
                    sender: context.id.to_owned(),
                    state: data.clone(),
                },
            );
            let _ = context.state.write_with(data);
        }
    });
}
