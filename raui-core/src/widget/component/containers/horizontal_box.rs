use crate::{
    widget,
    widget::{
        component::containers::flex_box::{flex_box, FlexBoxProps},
        unit::flex::FlexBoxDirection,
        utils::Transform,
    },
    widget_component, Scalar,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct HorizontalBoxProps {
    #[serde(default)]
    pub separation: Scalar,
    #[serde(default)]
    pub reversed: bool,
    #[serde(default)]
    pub transform: Transform,
}
implement_props_data!(HorizontalBoxProps);

widget_component! {
    pub horizontal_box(key, props, listed_slots) {
        let HorizontalBoxProps { separation, reversed, transform } = props.read_cloned_or_default();
        let props = props.clone().with(FlexBoxProps {
            direction: if reversed {
                FlexBoxDirection::HorizontalRightToLeft
            } else {
                FlexBoxDirection::HorizontalLeftToRight
            },
            separation,
            wrap: false,
            transform,
        });

        widget! {
            (#{key} flex_box: {props} |[ listed_slots ]|)
        }
    }
}
