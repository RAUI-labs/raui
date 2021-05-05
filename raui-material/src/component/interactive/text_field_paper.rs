use crate::{
    component::{
        containers::paper::{paper, PaperProps},
        text_paper::{text_paper, TextPaperProps},
    },
    theme::ThemedWidgetProps,
};
use raui_core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Clone, Serialize, Deserialize)]
#[props_data(raui_core::props::PropsData)]
#[prefab(raui_core::Prefab)]
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
    #[serde(default = "TextFieldPaperProps::default_inactive_alpha")]
    pub inactive_alpha: Scalar,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub horizontal_align_override: Option<TextBoxHorizontalAlign>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_align_override: Option<TextBoxVerticalAlign>,
    #[serde(default)]
    pub transform: Transform,
    #[serde(default)]
    pub paper_theme: ThemedWidgetProps,
    #[serde(default = "TextFieldPaperProps::default_padding")]
    pub padding: Rect,
    #[serde(default)]
    pub password: Option<char>,
}

impl TextFieldPaperProps {
    fn default_inactive_alpha() -> Scalar {
        0.75
    }

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
            inactive_alpha: Self::default_inactive_alpha(),
            horizontal_align_override: Default::default(),
            vertical_align_override: Default::default(),
            transform: Default::default(),
            paper_theme: Default::default(),
            padding: Self::default_padding(),
            password: Default::default(),
        }
    }
}

fn text_field_paper_content(context: WidgetContext) -> WidgetNode {
    let WidgetContext { key, props, .. } = context;

    let TextFieldPaperProps {
        hint,
        width,
        height,
        variant,
        use_main_color,
        inactive_alpha,
        horizontal_align_override,
        vertical_align_override,
        transform,
        paper_theme,
        padding,
        password,
    } = props.read_cloned_or_default();
    let TextInputProps {
        text,
        cursor_position,
        focused,
        ..
    } = props.read_cloned_or_default();
    let text = if let Some(c) = password {
        std::iter::repeat(c).take(text.len()).collect()
    } else {
        text.trim().to_owned()
    };
    let text = if text.is_empty() {
        hint
    } else if focused {
        if cursor_position < text.len() {
            format!("{}|{}", &text[..cursor_position], &text[cursor_position..])
        } else {
            format!("{}|", text)
        }
    } else {
        text.to_owned()
    };
    let paper_variant = props.map_or_default::<PaperProps, _, _>(|p| p.variant.clone());
    let paper_props = props
        .clone()
        .with(PaperProps {
            variant: paper_variant,
            ..Default::default()
        })
        .with(paper_theme);
    let text_props = props
        .clone()
        .with(TextPaperProps {
            text,
            width,
            height,
            variant,
            use_main_color,
            horizontal_align_override,
            vertical_align_override,
            transform,
        })
        .with(ContentBoxItemLayout {
            margin: padding,
            ..Default::default()
        });
    let alpha = if focused { 1.0 } else { inactive_alpha };

    widget! {
        (#{key} paper: {paper_props} [
            (#{"text"} text_paper: {text_props} | {WidgetAlpha(alpha)})
        ])
    }
}

pub fn text_field_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        idref, key, props, ..
    } = context;

    widget! {
        (#{key} | {idref.cloned()} input_field: {props.clone()} {
            content = (#{"content"} text_field_paper_content: {props.clone()})
        })
    }
}
