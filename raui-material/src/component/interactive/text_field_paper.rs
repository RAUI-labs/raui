use crate::{
    component::{
        containers::paper::{PaperProps, paper},
        text_paper::{TextPaperProps, text_paper},
    },
    theme::ThemedWidgetProps,
};
use raui_core::{
    PropsData, Scalar, make_widget,
    widget::{
        component::{
            WidgetAlpha, WidgetComponent,
            interactive::input_field::{
                TextInputProps, TextInputState, input_field, input_text_with_cursor,
            },
        },
        context::WidgetContext,
        node::WidgetNode,
        unit::{
            content::ContentBoxItemLayout,
            text::{TextBoxHorizontalAlign, TextBoxSizeValue, TextBoxVerticalAlign},
        },
        utils::{Color, Rect, Transform},
    },
};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_override: Option<Color>,
    #[serde(default)]
    pub transform: Transform,
    #[serde(default)]
    pub paper_theme: ThemedWidgetProps,
    #[serde(default = "TextFieldPaperProps::default_padding")]
    pub padding: Rect,
    #[serde(default)]
    pub password: Option<char>,
    #[serde(default = "TextFieldPaperProps::default_cursor")]
    pub cursor: Option<char>,
}

impl TextFieldPaperProps {
    fn default_inactive_alpha() -> Scalar {
        0.75
    }

    fn default_cursor() -> Option<char> {
        Some('|')
    }

    fn default_padding() -> Rect {
        4.0.into()
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
            color_override: Default::default(),
            transform: Default::default(),
            paper_theme: Default::default(),
            padding: Self::default_padding(),
            password: Default::default(),
            cursor: Self::default_cursor(),
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
        color_override,
        transform,
        paper_theme,
        padding,
        password,
        cursor,
    } = props.read_cloned_or_default();
    let TextInputState {
        cursor_position,
        focused,
    } = props.read_cloned_or_default();
    let text = props
        .read::<TextInputProps>()
        .ok()
        .and_then(|props| props.text.as_ref())
        .map(|text| text.get())
        .unwrap_or_default();
    let text = if let Some(c) = password {
        std::iter::repeat_n(c, text.chars().count()).collect()
    } else {
        text
    };
    let text = if focused {
        if let Some(cursor) = cursor {
            input_text_with_cursor(&text, cursor_position, cursor)
        } else {
            text
        }
    } else if text.is_empty() {
        hint
    } else {
        text
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
            color_override,
            transform,
        })
        .with(ContentBoxItemLayout {
            margin: padding,
            ..Default::default()
        });
    let alpha = if focused { 1.0 } else { inactive_alpha };

    make_widget!(paper)
        .key(key)
        .merge_props(paper_props)
        .listed_slot(
            make_widget!(text_paper)
                .key("text")
                .merge_props(text_props)
                .with_shared_props(WidgetAlpha(alpha)),
        )
        .into()
}

pub fn text_field_paper(context: WidgetContext) -> WidgetNode {
    text_field_paper_impl(make_widget!(input_field), context)
}

pub fn text_field_paper_impl(component: WidgetComponent, context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        idref, key, props, ..
    } = context;

    component
        .key(key)
        .maybe_idref(idref.cloned())
        .merge_props(props.clone())
        .named_slot(
            "content",
            make_widget!(text_field_paper_content)
                .key("text")
                .merge_props(props.clone()),
        )
        .into()
}
