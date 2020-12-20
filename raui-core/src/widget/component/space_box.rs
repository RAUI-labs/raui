use crate::{
    widget,
    widget::unit::size::{SizeBoxNode, SizeBoxSizeValue},
    widget_component, Scalar,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct SpaceBoxProps {
    #[serde(default)]
    pub width: Scalar,
    #[serde(default)]
    pub height: Scalar,
}
implement_props_data!(SpaceBoxProps, "SpaceBoxProps");

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
