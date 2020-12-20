use crate::ui::components::button_state_image::button_state_image;
use raui_core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ImageButtonProps {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub horizontal_alignment: Scalar,
}
implement_props_data!(ImageButtonProps, "ImageButtonProps");

widget_component! {
    pub image_button(key, props) {
        widget! {
            (#{key} button: {props.clone()} {
                content = (button_state_image: {props.clone()})
            })
        }
    }
}
