use crate::component::containers::paper::paper;
use raui_core::{
    make_widget,
    widget::{
        component::containers::grid_box::{grid_box, nav_grid_box},
        context::WidgetContext,
        node::WidgetNode,
        unit::content::ContentBoxItemLayout,
    },
};

pub fn nav_grid_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        idref,
        key,
        props,
        listed_slots,
        ..
    } = context;

    let inner_props = props.clone().without::<ContentBoxItemLayout>();

    make_widget!(paper)
        .key(key)
        .maybe_idref(idref.cloned())
        .merge_props(props.clone())
        .listed_slot(
            make_widget!(nav_grid_box)
                .key("grid")
                .merge_props(inner_props)
                .listed_slots(listed_slots),
        )
        .into()
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

    make_widget!(paper)
        .key(key)
        .maybe_idref(idref.cloned())
        .merge_props(props.clone())
        .listed_slot(
            make_widget!(grid_box)
                .key("grid")
                .merge_props(inner_props)
                .listed_slots(listed_slots),
        )
        .into()
}
