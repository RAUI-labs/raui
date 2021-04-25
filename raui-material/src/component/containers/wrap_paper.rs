use crate::component::containers::paper::paper;
use raui_core::prelude::*;

pub fn wrap_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        idref,
        key,
        props,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => content);

    widget! {
        (#{key} | {idref.cloned()} paper: {props.clone()} [
            (#{"wrap"} wrap_box: {props.clone()} {
                content = {content}
            })
        ])
    }
}
