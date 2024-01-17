use crate::theme::{ThemeColor, ThemeProps, ThemedTextMaterial, ThemedWidgetProps};
use raui_core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(raui_core::props::PropsData)]
#[prefab(raui_core::Prefab)]
pub struct TextPaperProps {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub width: TextBoxSizeValue,
    #[serde(default)]
    pub height: TextBoxSizeValue,
    #[serde(default)]
    pub variant: String,
    #[serde(default)]
    pub use_main_color: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub horizontal_align_override: Option<TextBoxHorizontalAlign>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_align_override: Option<TextBoxVerticalAlign>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_override: Option<Color>,
    #[serde(default)]
    pub transform: Transform,
}

pub fn text_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        idref,
        key,
        props,
        shared_props,
        ..
    } = context;

    let TextPaperProps {
        text,
        width,
        height,
        variant,
        use_main_color,
        horizontal_align_override,
        vertical_align_override,
        color_override,
        transform,
    } = props.read_cloned_or_default();
    let themed_props = props.read_cloned_or_default::<ThemedWidgetProps>();
    let ThemedTextMaterial {
        mut horizontal_align,
        mut vertical_align,
        direction,
        font,
    } = match shared_props.read::<ThemeProps>() {
        Ok(props) => props
            .text_variants
            .get(&variant)
            .cloned()
            .unwrap_or_default(),
        Err(_) => Default::default(),
    };
    if let Some(horizontal_override) = horizontal_align_override {
        horizontal_align = horizontal_override;
    }
    if let Some(alignment_override) = vertical_align_override {
        vertical_align = alignment_override;
    }
    let color = if let Some(color_override) = color_override {
        color_override
    } else {
        match shared_props.read::<ThemeProps>() {
            Ok(props) => {
                if use_main_color {
                    match themed_props.color {
                        ThemeColor::Default => props.active_colors.main.default.main,
                        ThemeColor::Primary => props.active_colors.main.primary.main,
                        ThemeColor::Secondary => props.active_colors.main.secondary.main,
                    }
                } else {
                    match themed_props.color {
                        ThemeColor::Default => props.active_colors.contrast.default.main,
                        ThemeColor::Primary => props.active_colors.contrast.primary.main,
                        ThemeColor::Secondary => props.active_colors.contrast.secondary.main,
                    }
                }
            }
            Err(_) => Default::default(),
        }
    };
    let props = TextBoxProps {
        text,
        width,
        height,
        horizontal_align,
        vertical_align,
        direction,
        font,
        color,
        transform,
    };

    make_widget!(text_box)
        .key(key)
        .maybe_idref(idref.cloned())
        .with_props(props)
        .into()
}
