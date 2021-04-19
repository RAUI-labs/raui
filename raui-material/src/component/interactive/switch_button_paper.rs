use crate::component::{interactive::button_paper::button_paper, switch_paper::switch_paper};
use raui_core::prelude::*;

pub fn switch_button_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext { key, props, .. } = context;

    widget! {
        (#{key} button_paper: {props.clone()} {
            content = (#{"switch"} switch_paper: {props.clone()})
        })
    }
}
