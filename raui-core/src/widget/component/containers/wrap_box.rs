use crate::{
    unpack_named_slots,
    widget::{
        context::WidgetContext,
        node::WidgetNode,
        unit::size::{SizeBoxNode, SizeBoxSizeValue},
        utils::Rect,
    },
    PropsData,
};
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct WrapBoxProps {
    #[serde(default)]
    pub margin: Rect,
    #[serde(default = "WrapBoxProps::default_fill")]
    pub fill: bool,
}

impl Default for WrapBoxProps {
    fn default() -> Self {
        Self {
            margin: Default::default(),
            fill: Self::default_fill(),
        }
    }
}

impl WrapBoxProps {
    fn default_fill() -> bool {
        true
    }
}

pub fn wrap_box(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        props,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => content);

    let WrapBoxProps { margin, fill } = props.read_cloned_or_default();
    let (width, height) = if fill {
        (SizeBoxSizeValue::Fill, SizeBoxSizeValue::Fill)
    } else {
        (SizeBoxSizeValue::Content, SizeBoxSizeValue::Content)
    };

    SizeBoxNode {
        id: id.to_owned(),
        props: props.clone(),
        slot: Box::new(content),
        margin,
        width,
        height,
        ..Default::default()
    }
    .into()
}
