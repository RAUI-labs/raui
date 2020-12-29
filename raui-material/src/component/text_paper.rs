use crate::theme::{ThemeColor, ThemeProps, ThemedTextMaterial, ThemedWidgetProps};
use raui_core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TextPaperProps {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub width: TextBoxSizeValue,
    #[serde(default)]
    pub height: TextBoxSizeValue,
    #[serde(default)]
    pub variant: String,
}
implement_props_data!(TextPaperProps, "TextPaperProps");

widget_component! {
    pub text_paper(key, props, shared_props) {
        let TextPaperProps { text, width, height, variant } = props.read_cloned_or_default();
        let themed_props = props.read_cloned_or_default::<ThemedWidgetProps>();
        let ThemedTextMaterial { alignment, direction, font } = match shared_props.read::<ThemeProps>() {
            Ok(props) => props
                .text_variants
                .get(&variant)
                .cloned()
                .unwrap_or_default(),
            Err(_) => Default::default(),
        };
        let color = match shared_props.read::<ThemeProps>() {
            Ok(props) => match themed_props.color {
                ThemeColor::Default => props.active_colors.contrast.default.main,
                ThemeColor::Primary => props.active_colors.contrast.primary.main,
                ThemeColor::Secondary => props.active_colors.contrast.secondary.main,
            },
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
        };

        widget! {
            (#{key} text_box: {props})
        }
    }
}
