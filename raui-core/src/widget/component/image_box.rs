use crate::{
    widget,
    widget::{
        unit::image::{ImageBoxAspectRatio, ImageBoxMaterial, ImageBoxNode, ImageBoxSizeValue},
        utils::Transform,
    },
    widget_component,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ImageBoxProps {
    #[serde(default)]
    pub width: ImageBoxSizeValue,
    #[serde(default)]
    pub height: ImageBoxSizeValue,
    #[serde(default)]
    pub content_keep_aspect_ratio: Option<ImageBoxAspectRatio>,
    #[serde(default)]
    pub material: ImageBoxMaterial,
    #[serde(default)]
    pub transform: Transform,
}
implement_props_data!(ImageBoxProps, "ImageBoxProps");

widget_component! {
    pub image_box(id, props) {
        let ImageBoxProps {
            width,
            height,
            content_keep_aspect_ratio,
            material,
            transform,
        } = props.read_cloned_or_default();

        widget! {{{
            ImageBoxNode {
                id: id.to_owned(),
                props: props.clone(),
                width,
                height,
                content_keep_aspect_ratio,
                material,
                transform,
            }
        }}}
    }
}
