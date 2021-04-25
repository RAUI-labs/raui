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
    pub alignment_override: Option<TextBoxAlignment>,
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
        alignment_override,
        transform,
    } = props.read_cloned_or_default();
    let themed_props = props.read_cloned_or_default::<ThemedWidgetProps>();
    let ThemedTextMaterial {
        mut alignment,
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
    if let Some(alignment_override) = alignment_override {
        alignment = alignment_override;
    }
    let color = match shared_props.read::<ThemeProps>() {
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
    };
    let props = TextBoxProps {
        text,
        width,
        height,
        alignment,
        direction,
        font,
        color,
        transform,
    };

    widget! {
        (#{key} | {idref.cloned()} text_box: {props})
    }
}
