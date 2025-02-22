use crate::{
    unpack_named_slots,
    widget::{
        context::WidgetContext,
        node::WidgetNode,
        unit::size::{SizeBoxAspectRatio, SizeBoxNode, SizeBoxSizeValue},
        utils::{Rect, Transform},
    },
    PropsData,
};
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct SizeBoxProps {
    #[serde(default)]
    pub width: SizeBoxSizeValue,
    #[serde(default)]
    pub height: SizeBoxSizeValue,
    #[serde(default)]
    pub margin: Rect,
    #[serde(default)]
    pub keep_aspect_ratio: SizeBoxAspectRatio,
    #[serde(default)]
    pub transform: Transform,
}

pub fn size_box(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        props,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => content);

    let SizeBoxProps {
        width,
        height,
        margin,
        keep_aspect_ratio,
        transform,
    } = props.read_cloned_or_default();

    SizeBoxNode {
        id: id.to_owned(),
        props: props.clone(),
        slot: Box::new(content),
        width,
        height,
        margin,
        keep_aspect_ratio,
        transform,
    }
    .into()
}
