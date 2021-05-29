use crate::component::containers::paper::paper;
use raui_core::prelude::*;

pub fn nav_flex_paper(context: WidgetContext) -> WidgetNode {
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
            (#{"flex"} nav_flex_box: {inner_props} |[ listed_slots ]|)
        ])
    }
}

pub fn flex_paper(context: WidgetContext) -> WidgetNode {
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
            (#{"flex"} flex_box: {inner_props} |[ listed_slots ]|)
        ])
    }
}
