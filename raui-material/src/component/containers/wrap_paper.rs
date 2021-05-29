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

    let inner_props = props.clone().without::<ContentBoxItemLayout>();

    widget! {
        (#{key} | {idref.cloned()} paper: {props.clone()} [
            (#{"wrap"} wrap_box: {inner_props} {
                content = {content}
            })
        ])
    }
}
