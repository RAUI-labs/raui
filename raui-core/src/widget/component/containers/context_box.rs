use crate::{
    PropsData, make_widget, pre_hooks, unpack_named_slots,
    widget::{
        component::containers::{
            anchor_box::{AnchorProps, PivotBoxProps, pivot_to_anchor_and_align, use_anchor_box},
            content_box::content_box,
            portal_box::{portal_box, use_portals_container_relative_layout},
        },
        context::WidgetContext,
        node::WidgetNode,
        unit::{area::AreaBoxNode, content::ContentBoxItemLayout},
        utils::{Rect, Vec2},
    },
};
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Copy, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct ContextBoxProps {
    #[serde(default)]
    pub show: bool,
}

#[pre_hooks(use_anchor_box)]
pub fn context_box(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        idref,
        key,
        props,
        state,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => {content, context, backdrop});

    let ContextBoxProps { show } = props.read_cloned_or_default();
    let anchor_state = state.read_cloned_or_default::<AnchorProps>();
    let pivot_props = props.read_cloned_or_default::<PivotBoxProps>();
    let (Vec2 { x, y }, align) = pivot_to_anchor_and_align(&pivot_props, &anchor_state);

    let context = if show {
        context.remap_props(|content_props| {
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

        make_widget!(portal_box)
            .named_slot(
                "content",
                make_widget!(content_box)
                    .key("content")
                    .listed_slot(backdrop)
                    .listed_slot(context),
            )
            .into()
    } else {
        WidgetNode::default()
    };

    let content = make_widget!(content_box)
        .key(key)
        .maybe_idref(idref.cloned())
        .merge_props(props.clone())
        .listed_slot(content)
        .listed_slot(context)
        .into();

    AreaBoxNode {
        id: id.to_owned(),
        slot: Box::new(content),
    }
    .into()
}

#[pre_hooks(use_portals_container_relative_layout)]
pub fn portals_context_box(mut context: WidgetContext) -> WidgetNode {
    context_box(context)
}
