use crate::component::containers::paper::paper;
use raui_core::prelude::*;

pub fn nav_horizontal_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        idref,
        key,
        props,
        listed_slots,
        ..
    } = context;

    let inner_props = props.clone().without::<ContentBoxItemLayout>();

    widget! {
        (#{key} | {idref.cloned()} paper: {props.clone()} [
            (#{"horizontal"} nav_horizontal_box: {inner_props} |[ listed_slots ]|)
        ])
    }
}

pub fn horizontal_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        idref,
        key,
        props,
        listed_slots,
        ..
    } = context;

    let inner_props = props.clone().without::<ContentBoxItemLayout>();

    widget! {
        (#{key} | {idref.cloned()} paper: {props.clone()} [
            (#{"horizontal"} horizontal_box: {inner_props} |[ listed_slots ]|)
        ])
    }
}
