use crate::{
    widget,
    widget::unit::size::{SizeBoxNode, SizeBoxSizeValue},
    widget_component, Scalar,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SpaceBoxProps {
    #[serde(default)]
    pub width: Scalar,
    #[serde(default)]
    pub height: Scalar,
}
implement_props_data!(SpaceBoxProps);

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

widget_component! {
    pub space_box(id, props) {
        let SpaceBoxProps { width, height } = props.read_cloned_or_default();

        widget! {{{
            SizeBoxNode {
                id: id.to_owned(),
                props: props.clone(),
                width: SizeBoxSizeValue::Exact(width),
                height: SizeBoxSizeValue::Exact(height),
                ..Default::default()
            }
        }}}
    }
}
