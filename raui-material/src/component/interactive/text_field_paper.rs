use crate::{
    component::{
        containers::paper::{paper, PaperProps},
        text_paper::{text_paper, TextPaperProps},
    },
    theme::ThemedWidgetProps,
};
use raui_core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextFieldPaperProps {
    #[serde(default)]
    pub hint: String,
    #[serde(default)]
    pub width: TextBoxSizeValue,
    #[serde(default)]
    pub height: TextBoxSizeValue,
    #[serde(default)]
    pub variant: String,
    #[serde(default)]
    pub use_main_color: bool,
    #[serde(default)]
    pub paper_theme: ThemedWidgetProps,
    #[serde(default = "TextFieldPaperProps::default_padding")]
    pub padding: Rect,
}
implement_props_data!(TextFieldPaperProps, "TextFieldPaperProps");

impl TextFieldPaperProps {
    fn default_padding() -> Rect {
        Rect {
            left: 4.0,
            right: 4.0,
            top: 4.0,
            bottom: 4.0,
        }
    }
}

impl Default for TextFieldPaperProps {
    fn default() -> Self {
        Self {
            hint: Default::default(),
            width: Default::default(),
            height: Default::default(),
            variant: Default::default(),
            use_main_color: Default::default(),
            paper_theme: Default::default(),
            padding: Self::default_padding(),
        }
    }
}

widget_component! {
    text_field_content(key, props) {
        let ButtonProps { selected, .. } = props.read_cloned_or_default();
        let TextFieldPaperProps {
            hint,
            width,
            height,
            variant,
            use_main_color,
            paper_theme,
            padding,
        } = props.read_cloned_or_default();
        let InputFieldProps { text, cursor_position, .. } = props.read_cloned_or_default();
        let text = text.trim();
        let text = if text.is_empty() {
            hint
        } else if selected {
            if cursor_position < text.len() {
                format!("{}|{}", &text[..cursor_position], &text[cursor_position..])
            } else {
                format!("{}|", text)
            }
        } else {
            text.to_owned()
        };
        let paper_variant = props.map_or_default::<PaperProps, _, _>(|p| p.variant.clone());
        let paper_props = props.clone().with(PaperProps {
            variant: paper_variant,
            ..Default::default()
        }).with(paper_theme);
        let props = props.clone().with(TextPaperProps {
            text,
            width,
            height,
            variant,
            use_main_color,
        }).with(ContentBoxItemLayout {
            margin: padding,
            ..Default::default()
        });

        widget! {
            (#{key} paper: {paper_props} [
                (#{key} text_paper: {props})
            ])
        }
    }
}

widget_component! {
    pub text_field_paper(key, props) {
        widget! {
            (#{key} input_field: {props.clone()} {
                content = (#{"text"} text_field_content: {props.clone()})
            })
        }
    }
}
