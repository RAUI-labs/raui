use crate::component::{containers::tooltip_paper::tooltip_paper, text_paper::text_paper};
use raui_core::prelude::*;

pub fn text_tooltip_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        idref,
        key,
        props,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => content);

    widget! {
        (#{key} | {idref.cloned()} tooltip_paper: {props.clone()} {
            content = {content}
            tooltip = (#{"text"} text_paper: {props.clone()})
        })
    }
}
