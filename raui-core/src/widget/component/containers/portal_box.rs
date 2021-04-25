use crate::{
    unpack_named_slots, widget,
    widget::{
        context::WidgetContext,
        node::WidgetNode,
        unit::{
            content::{ContentBoxItemLayout, ContentBoxItemNode},
            flex::{FlexBoxItemLayout, FlexBoxItemNode},
            grid::{GridBoxItemLayout, GridBoxItemNode},
            portal::{PortalBoxNode, PortalBoxSlotNode},
        },
        WidgetRef,
    },
    PropsData,
};
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct PortalBoxProps(#[serde(default)] pub WidgetRef);

pub fn portal_box(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        props,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => content);

    let PortalBoxProps(owner) = props.read_cloned_or_default();
    let slot = if let Ok(layout) = props.read_cloned::<ContentBoxItemLayout>() {
        PortalBoxSlotNode::ContentItem(ContentBoxItemNode {
            slot: content,
            layout,
        })
    } else if let Ok(layout) = props.read_cloned::<FlexBoxItemLayout>() {
        PortalBoxSlotNode::FlexItem(FlexBoxItemNode {
            slot: content,
            layout,
        })
    } else if let Ok(layout) = props.read_cloned::<GridBoxItemLayout>() {
        PortalBoxSlotNode::GridItem(GridBoxItemNode {
            slot: content,
            layout,
        })
    } else {
        PortalBoxSlotNode::Slot(content)
    };

    if let Some(owner) = owner.read() {
        widget! {{{
            PortalBoxNode {
                id: id.to_owned(),
                slot: Box::new(slot),
                owner,
            }
        }}}
    } else {
        widget! {()}
    }
}
