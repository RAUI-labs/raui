use crate::component::containers::paper::paper;
use raui_core::prelude::*;

pub fn nav_vertical_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        props,
        listed_slots,
        ..
    } = context;

    widget! {
        (#{key} paper: {props.clone()} [
            (#{"vertical"} nav_vertical_box: {props.clone()} |[ listed_slots ]|)
        ])
    }
}

pub fn vertical_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        props,
        listed_slots,
        ..
    } = context;

    widget! {
        (#{key} paper: {props.clone()} [
            (#{"vertical"} vertical_box: {props.clone()} |[ listed_slots ]|)
        ])
    }
}
