use crate::{
    messenger::MessageData,
    pre_hooks, unpack_named_slots, widget,
    widget::{
        component::{use_relative_layout_listener, RelativeLayoutListenerSignal},
        context::{WidgetContext, WidgetMountOrChangeContext},
        node::WidgetNode,
        unit::{area::AreaBoxNode, content::ContentBoxItemLayout},
        utils::{lerp, Rect, Vec2},
        WidgetId, WidgetIdOrRef,
    },
    MessageData, PropsData,
};
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Copy, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct AnchorProps {
    #[serde(default)]
    pub outer_box_size: Vec2,
    #[serde(default)]
    pub inner_box_rect: Rect,
}

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct AnchorNotifyProps(
    #[serde(default)]
    #[serde(skip_serializing_if = "WidgetIdOrRef::is_none")]
    pub WidgetIdOrRef,
);

#[derive(MessageData, Debug, Clone)]
#[message_data(crate::messenger::MessageData)]
pub struct AnchorNotifyMessage {
    pub sender: WidgetId,
    pub state: AnchorProps,
    pub prev: AnchorProps,
}

#[derive(PropsData, Debug, Copy, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct PivotBoxProps {
    #[serde(default = "PivotBoxProps::default_pivot")]
    pub pivot: Vec2,
    #[serde(default)]
    pub align: Vec2,
}

impl Default for PivotBoxProps {
    fn default() -> Self {
        Self {
            pivot: Self::default_pivot(),
            align: Default::default(),
        }
    }
}

impl PivotBoxProps {
    fn default_pivot() -> Vec2 {
        Vec2 { x: 0.0, y: 1.0 }
    }
}

pub fn use_anchor_box_notified_state(context: &mut WidgetContext) {
    context.life_cycle.change(|context| {
        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<AnchorNotifyMessage>() {
                let _ = context.state.write_with(msg.state);
            }
        }
    });
}

#[pre_hooks(use_relative_layout_listener)]
pub fn use_anchor_box(context: &mut WidgetContext) {
    fn notify<T>(context: &WidgetMountOrChangeContext, data: T)
    where
        T: 'static + MessageData,
    {
        if let Ok(AnchorNotifyProps(notify)) = context.props.read() {
            if let Some(to) = notify.read() {
                context.messenger.write(to, data);
            }
        }
    }

    context.life_cycle.mount(|context| {
        notify(
            &context,
            AnchorNotifyMessage {
                sender: context.id.to_owned(),
                state: Default::default(),
                prev: Default::default(),
            },
        );
        let _ = context.state.write_with(AnchorProps::default());
    });

    context.life_cycle.change(|context| {
        let mut data = context.state.read_cloned_or_default::<AnchorProps>();
        let prev = data;
        let mut dirty = false;
        for msg in context.messenger.messages {
            if let Some(RelativeLayoutListenerSignal::Change(size, rect)) =
                msg.as_any().downcast_ref()
            {
                data.outer_box_size = *size;
                data.inner_box_rect = *rect;
                dirty = true;
            }
        }
        if dirty {
            notify(
                &context,
                AnchorNotifyMessage {
                    sender: context.id.to_owned(),
                    state: data,
                    prev,
                },
            );
            let _ = context.state.write_with(data);
        }
    });
}

#[pre_hooks(use_anchor_box)]
pub fn anchor_box(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        state,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => content);

    let anchor_props = state.read_cloned_or_default::<AnchorProps>();

    content.remap_props(|props| props.with(anchor_props));

    widget! {{{
        AreaBoxNode {
            id: id.to_owned(),
            slot: Box::new(content),
        }
    }}}
}

pub fn pivot_point_to_anchor(pivot: Vec2, anchor: &AnchorProps) -> Vec2 {
    let x = if anchor.outer_box_size.x > 0.0 {
        let v = lerp(
            anchor.inner_box_rect.left,
            anchor.inner_box_rect.right,
            pivot.x,
        );
        v / anchor.outer_box_size.x
    } else {
        0.0
    };
    let y = if anchor.outer_box_size.y > 0.0 {
        let v = lerp(
            anchor.inner_box_rect.top,
            anchor.inner_box_rect.bottom,
            pivot.y,
        );
        v / anchor.outer_box_size.y
    } else {
        0.0
    };
    Vec2 { x, y }
}

/// (anchor point, align factor)
pub fn pivot_to_anchor_and_align(pivot: &PivotBoxProps, anchor: &AnchorProps) -> (Vec2, Vec2) {
    (pivot_point_to_anchor(pivot.pivot, anchor), pivot.align)
}

pub fn pivot_box(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        props,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => content);

    let anchor_props = props.read_cloned_or_default::<AnchorProps>();
    let pivot_props = props.read_cloned_or_default::<PivotBoxProps>();
    let (Vec2 { x, y }, align) = pivot_to_anchor_and_align(&pivot_props, &anchor_props);

    content.remap_props(|content_props| {
        let mut item_props = content_props.read_cloned_or_default::<ContentBoxItemLayout>();
        item_props.anchors = Rect {
            left: x,
            right: x,
            top: y,
            bottom: y,
        };
        item_props.align = align;
        content_props.with(item_props)
    });

    widget! {{{
        AreaBoxNode {
            id: id.to_owned(),
            slot: Box::new(content),
        }
    }}}
}
