use crate::component::{interactive::button_paper::button_paper, text_paper::text_paper};
use raui_core::prelude::*;

pub fn text_button_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext { key, props, .. } = context;

    widget! {
        (#{key} button_paper: {props.clone()} {
            content = (#{"text"} text_paper: {props.clone()})
        })
    }
}
