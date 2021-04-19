use crate::component::containers::paper::paper;
use raui_core::prelude::*;

pub fn nav_grid_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        props,
        listed_slots,
        ..
    } = context;

    widget! {
        (#{key} paper: {props.clone()} [
            (#{"grid"} nav_grid_box: {props.clone()} |[ listed_slots ]|)
        ])
    }
}

pub fn grid_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        props,
        listed_slots,
        ..
    } = context;

    widget! {
        (#{key} paper: {props.clone()} [
            (#{"grid"} grid_box: {props.clone()} |[ listed_slots ]|)
        ])
    }
}
