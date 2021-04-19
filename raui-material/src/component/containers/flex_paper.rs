use crate::component::containers::paper::paper;
use raui_core::prelude::*;

pub fn nav_flex_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        props,
        listed_slots,
        ..
    } = context;

    widget! {
        (#{key} paper: {props.clone()} [
            (#{"flex"} nav_flex_box: {props.clone()} |[ listed_slots ]|)
        ])
    }
}

pub fn flex_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        props,
        listed_slots,
        ..
    } = context;

    widget! {
        (#{key} paper: {props.clone()} [
            (#{"flex"} flex_box: {props.clone()} |[ listed_slots ]|)
        ])
    }
}
