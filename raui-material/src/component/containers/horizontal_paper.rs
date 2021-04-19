use crate::component::containers::paper::paper;
use raui_core::prelude::*;

pub fn nav_horizontal_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        props,
        listed_slots,
        ..
    } = context;

    widget! {
        (#{key} paper: {props.clone()} [
            (#{"horizontal"} nav_horizontal_box: {props.clone()} |[ listed_slots ]|)
        ])
    }
}

pub fn horizontal_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        props,
        listed_slots,
        ..
    } = context;

    widget! {
        (#{key} paper: {props.clone()} [
            (#{"horizontal"} horizontal_box: {props.clone()} |[ listed_slots ]|)
        ])
    }
}
