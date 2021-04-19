use crate::{
    widget,
    widget::{
        component::WidgetAlpha,
        context::WidgetContext,
        node::WidgetNode,
        unit::text::{
            TextBoxAlignment, TextBoxDirection, TextBoxFont, TextBoxNode, TextBoxSizeValue,
        },
        utils::{Color, Transform},
    },
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
implement_props_data!(TextBoxProps);

pub fn text_box(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        props,
        shared_props,
        ..
    } = context;

    let TextBoxProps {
        width,
        height,
        text,
        alignment,
        direction,
        font,
        mut color,
        transform,
    } = props.read_cloned_or_default();

    let alpha = shared_props.read_cloned_or_default::<WidgetAlpha>().0;
    color.a *= alpha;

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
