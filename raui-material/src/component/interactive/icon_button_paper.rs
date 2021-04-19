use crate::component::{icon_paper::icon_paper, interactive::button_paper::button_paper};
use raui_core::prelude::*;

pub fn icon_button_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext { key, props, .. } = context;

    widget! {
        (#{key} button_paper: {props.clone()} {
            content = (#{"icon"} icon_paper: {props.clone()})
        })
    }
}
