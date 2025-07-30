use crate::component::containers::paper::paper;
use raui_core::{
    make_widget,
    widget::{
        component::containers::vertical_box::{nav_vertical_box, vertical_box},
        context::WidgetContext,
        node::WidgetNode,
        unit::content::ContentBoxItemLayout,
    },
};

pub fn nav_vertical_paper(context: WidgetContext) -> WidgetNode {
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
            make_widget!(nav_vertical_box)
                .key("vertical")
                .merge_props(inner_props)
                .listed_slots(listed_slots),
        )
        .into()
}

pub fn vertical_paper(context: WidgetContext) -> WidgetNode {
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
            make_widget!(vertical_box)
                .key("vertical")
                .merge_props(inner_props)
                .listed_slots(listed_slots),
        )
        .into()
}
