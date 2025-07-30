use crate::component::containers::paper::paper;
use raui_core::{
    make_widget,
    widget::{
        component::containers::flex_box::{flex_box, nav_flex_box},
        context::WidgetContext,
        node::WidgetNode,
        unit::content::ContentBoxItemLayout,
    },
};

pub fn nav_flex_paper(context: WidgetContext) -> WidgetNode {
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
            make_widget!(nav_flex_box)
                .key("flex")
                .merge_props(inner_props)
                .listed_slots(listed_slots),
        )
        .into()
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

    make_widget!(paper)
        .key(key)
        .maybe_idref(idref.cloned())
        .merge_props(props.clone())
        .listed_slot(
            make_widget!(flex_box)
                .key("flex")
                .merge_props(inner_props)
                .listed_slots(listed_slots),
        )
        .into()
}
