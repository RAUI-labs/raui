use crate::{
    widget::{
        context::WidgetContext,
        node::WidgetNode,
        unit::size::{SizeBoxNode, SizeBoxSizeValue},
    },
    PropsData, Scalar,
};
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct SpaceBoxProps {
    #[serde(default)]
    pub width: Scalar,
    #[serde(default)]
    pub height: Scalar,
}

impl SpaceBoxProps {
    pub fn cube(value: Scalar) -> Self {
        Self {
            width: value,
            height: value,
        }
    }

    pub fn horizontal(width: Scalar) -> Self {
        Self { width, height: 0.0 }
    }

    pub fn vertical(height: Scalar) -> Self {
        Self { width: 0.0, height }
    }
}

pub fn space_box(context: WidgetContext) -> WidgetNode {
    let WidgetContext { id, props, .. } = context;

    let SpaceBoxProps { width, height } = props.read_cloned_or_default();

    SizeBoxNode {
        id: id.to_owned(),
        props: props.clone(),
        width: SizeBoxSizeValue::Exact(width),
        height: SizeBoxSizeValue::Exact(height),
        ..Default::default()
    }
    .into()
}
