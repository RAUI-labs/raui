use crate::theme::{ThemeColor, ThemeProps, ThemedWidgetProps};
use raui_core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct IconImage {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_rect: Option<Rect>,
    #[serde(default)]
    pub scaling: ImageBoxImageScaling,
}

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(raui_core::props::PropsData)]
#[prefab(raui_core::Prefab)]
pub struct IconPaperProps {
    #[serde(default)]
    pub image: IconImage,
    #[serde(default)]
    pub size_level: usize,
    #[serde(default)]
    pub transform: Transform,
}

pub fn icon_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        idref,
        key,
        props,
        shared_props,
        ..
    } = context;

    let themed_props = props.read_cloned_or_default::<ThemedWidgetProps>();
    let tint = match shared_props.read::<ThemeProps>() {
        Ok(props) => match themed_props.color {
            ThemeColor::Default => props.active_colors.contrast.default.main,
            ThemeColor::Primary => props.active_colors.contrast.primary.main,
            ThemeColor::Secondary => props.active_colors.contrast.secondary.main,
        },
        Err(_) => Default::default(),
    };
    let icon_props = props.read_cloned_or_default::<IconPaperProps>();
    let size = match shared_props.read::<ThemeProps>() {
        Ok(props) => props
            .icons_level_sizes
            .get(icon_props.size_level)
            .copied()
            .unwrap_or(24.0),
        Err(_) => 24.0,
    };
    let IconImage {
        id,
        source_rect,
        scaling,
    } = icon_props.image;
    let image = ImageBoxImage {
        id,
        source_rect,
        scaling,
        tint,
    };
    let props = ImageBoxProps {
        width: ImageBoxSizeValue::Exact(size),
        height: ImageBoxSizeValue::Exact(size),
        content_keep_aspect_ratio: Some(ImageBoxAspectRatio {
            horizontal_alignment: 0.5,
            vertical_alignment: 0.5,
            outside: false,
        }),
        material: ImageBoxMaterial::Image(image),
        transform: icon_props.transform,
    };

    make_widget!(image_box)
        .key(key)
        .maybe_idref(idref.cloned())
        .with_props(props)
        .into()
}
