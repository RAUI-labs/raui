use crate::{
    widget,
    widget::{
        component::WidgetAlpha,
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
implement_props_data!(TextBoxProps);

widget_component!(
    pub fn text_box(id: Id, props: Props, shared_props: SharedProps) {
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
);
