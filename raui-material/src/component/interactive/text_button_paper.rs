use crate::component::{interactive::button_paper::button_paper, text_paper::text_paper};
use raui_core::prelude::*;

pub fn text_button_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        idref, key, props, ..
    } = context;

    let wrap_props = props.read_cloned_or_default::<WrapBoxProps>();

    widget! {
        (#{key} | {idref.cloned()} button_paper: {props.clone()} {
            content = (#{"wrap"} wrap_box: {wrap_props} {
                content = (#{"text"} text_paper: {props.clone()})
            })
        })
    }
}
