use crate::component::containers::paper::paper;
use raui_core::prelude::*;

pub fn nav_grid_paper(context: WidgetContext) -> WidgetNode {
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
            (#{"grid"} nav_grid_box: {inner_props} |[ listed_slots ]|)
        ])
    }
}

pub fn grid_paper(context: WidgetContext) -> WidgetNode {
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
            (#{"grid"} grid_box: {inner_props} |[ listed_slots ]|)
        ])
    }
}
