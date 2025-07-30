use crate::{
    PropsData,
    widget::{
        component::WidgetAlpha,
        context::WidgetContext,
        node::WidgetNode,
        unit::image::{
            ImageBoxAspectRatio, ImageBoxColor, ImageBoxImage, ImageBoxMaterial, ImageBoxNode,
            ImageBoxSizeValue,
        },
        utils::{Color, Transform},
    },
};
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct ImageBoxProps {
    #[serde(default)]
    pub width: ImageBoxSizeValue,
    #[serde(default)]
    pub height: ImageBoxSizeValue,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_keep_aspect_ratio: Option<ImageBoxAspectRatio>,
    #[serde(default)]
    pub material: ImageBoxMaterial,
    #[serde(default)]
    pub transform: Transform,
}

impl ImageBoxProps {
    pub fn colored(color: Color) -> Self {
        Self {
            material: ImageBoxMaterial::Color(ImageBoxColor {
                color,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    pub fn image(id: impl ToString) -> Self {
        Self {
            material: ImageBoxMaterial::Image(ImageBoxImage {
                id: id.to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    pub fn image_aspect_ratio(id: impl ToString, outside: bool) -> Self {
        Self {
            material: ImageBoxMaterial::Image(ImageBoxImage {
                id: id.to_string(),
                ..Default::default()
            }),
            content_keep_aspect_ratio: Some(ImageBoxAspectRatio {
                horizontal_alignment: 0.5,
                vertical_alignment: 0.5,
                outside,
            }),
            ..Default::default()
        }
    }
}

pub fn image_box(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        props,
        shared_props,
        ..
    } = context;

    let ImageBoxProps {
        width,
        height,
        content_keep_aspect_ratio,
        mut material,
        transform,
    } = props.read_cloned_or_default();

    let alpha = shared_props.read_cloned_or_default::<WidgetAlpha>().0;
    match &mut material {
        ImageBoxMaterial::Color(image) => {
            image.color.a *= alpha;
        }
        ImageBoxMaterial::Image(image) => {
            image.tint.a *= alpha;
        }
        _ => {}
    }

    ImageBoxNode {
        id: id.to_owned(),
        props: props.clone(),
        width,
        height,
        content_keep_aspect_ratio,
        material,
        transform,
    }
    .into()
}
