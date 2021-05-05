use crate::{
    widget,
    widget::{
        component::WidgetAlpha,
        context::WidgetContext,
        node::WidgetNode,
        unit::text::{
            TextBoxDirection, TextBoxFont, TextBoxHorizontalAlign, TextBoxNode, TextBoxSizeValue,
            TextBoxVerticalAlign,
        },
        utils::{Color, Transform},
    },
    PropsData,
};
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct TextBoxProps {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub width: TextBoxSizeValue,
    #[serde(default)]
    pub height: TextBoxSizeValue,
    #[serde(default)]
    pub horizontal_align: TextBoxHorizontalAlign,
    #[serde(default)]
    pub vertical_align: TextBoxVerticalAlign,
    #[serde(default)]
    pub direction: TextBoxDirection,
    #[serde(default)]
    pub font: TextBoxFont,
    #[serde(default)]
    pub color: Color,
    #[serde(default)]
    pub transform: Transform,
}

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
        horizontal_align,
        vertical_align,
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
            horizontal_align,
            vertical_align,
            direction,
            font,
            color,
            transform,
        }
    }}}
}
