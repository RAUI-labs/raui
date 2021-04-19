use crate::theme::{ThemeColor, ThemeProps, ThemedImageMaterial, ThemedWidgetProps};
use raui_core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SwitchPaperProps {
    #[serde(default)]
    pub on: bool,
    #[serde(default)]
    pub variant: String,
    #[serde(default)]
    pub size_level: usize,
}
implement_props_data!(SwitchPaperProps);

pub fn switch_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        props,
        shared_props,
        ..
    } = context;

    let SwitchPaperProps {
        on,
        variant,
        size_level,
    } = props.read_cloned_or_default();
    let themed_props = props.read_cloned_or_default::<ThemedWidgetProps>();
    let color = match shared_props.read::<ThemeProps>() {
        Ok(props) => match themed_props.color {
            ThemeColor::Default => props.active_colors.main.default.main,
            ThemeColor::Primary => props.active_colors.main.primary.main,
            ThemeColor::Secondary => props.active_colors.main.secondary.main,
        },
        Err(_) => Default::default(),
    };
    let (size, material) = match shared_props.read::<ThemeProps>() {
        Ok(props) => {
            let size = props
                .icons_level_sizes
                .get(size_level)
                .copied()
                .unwrap_or(24.0);
            let material = if let Some(material) = props.switch_variants.get(&variant) {
                if on {
                    material.on.clone()
                } else {
                    material.off.clone()
                }
            } else {
                Default::default()
            };
            (size, material)
        }
        Err(_) => (24.0, Default::default()),
    };
    let image = match material {
        ThemedImageMaterial::Color => ImageBoxProps {
            material: ImageBoxMaterial::Color(ImageBoxColor {
                color,
                scaling: if on {
                    ImageBoxImageScaling::Stretch
                } else {
                    ImageBoxImageScaling::Frame((size_level as Scalar, true).into())
                },
            }),
            width: ImageBoxSizeValue::Exact(size),
            height: ImageBoxSizeValue::Exact(size),
            ..Default::default()
        },
        ThemedImageMaterial::Image(mut data) => {
            data.tint = color;
            ImageBoxProps {
                material: ImageBoxMaterial::Image(data),
                width: ImageBoxSizeValue::Exact(size),
                height: ImageBoxSizeValue::Exact(size),
                ..Default::default()
            }
        }
        ThemedImageMaterial::Procedural(data) => ImageBoxProps {
            material: ImageBoxMaterial::Procedural(data),
            width: ImageBoxSizeValue::Exact(size),
            height: ImageBoxSizeValue::Exact(size),
            ..Default::default()
        },
    };
    widget! {
        (#{key} image_box: {image})
    }
}
