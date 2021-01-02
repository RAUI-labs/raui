use crate::{
    widget,
    widget::{
        unit::text::{
            TextBoxAlignment, TextBoxDirection, TextBoxFont, TextBoxNode, TextBoxSizeValue,
        },
        utils::{Color, Transform},
    },
    widget_component,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TextBoxProps {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub width: TextBoxSizeValue,
    #[serde(default)]
    pub height: TextBoxSizeValue,
    #[serde(default)]
    pub alignment: TextBoxAlignment,
    #[serde(default)]
    pub direction: TextBoxDirection,
    #[serde(default)]
    pub font: TextBoxFont,
    #[serde(default)]
    pub color: Color,
    #[serde(default)]
    pub transform: Transform,
}
implement_props_data!(TextBoxProps, "TextBoxProps");

widget_component! {
    pub text_box(id, props) {
        let TextBoxProps {
            width,
            height,
            text,
            alignment,
            direction,
            font,
            color,
            transform,
        } = props.read_cloned_or_default();

        widget! {{{
            TextBoxNode {
                id: id.to_owned(),
                props: props.clone(),
                text,
                width,
                height,
                alignment,
                direction,
                font,
                color,
                transform,
            }
        }}}
    }
}
