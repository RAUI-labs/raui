use crate::component::{icon_paper::icon_paper, interactive::button_paper::button_paper};
use raui_core::prelude::*;

pub fn icon_button_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        idref, key, props, ..
    } = context;

    widget! {
        (#{key} | {idref.cloned()} button_paper: {props.clone()} {
            content = (#{"icon"} icon_paper: {props.clone()})
        })
    }
}
