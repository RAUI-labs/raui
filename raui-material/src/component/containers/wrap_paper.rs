use crate::component::containers::paper::paper;
use raui_core::{
    make_widget, unpack_named_slots,
    widget::{
        component::containers::wrap_box::wrap_box, context::WidgetContext, node::WidgetNode,
        unit::content::ContentBoxItemLayout,
    },
};

pub fn wrap_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        idref,
        key,
        props,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => content);

    let inner_props = props.clone().without::<ContentBoxItemLayout>();

    make_widget!(paper)
        .key(key)
        .maybe_idref(idref.cloned())
        .merge_props(props.clone())
        .listed_slot(
            make_widget!(wrap_box)
                .key("wrap")
                .merge_props(inner_props)
                .named_slot("content", content),
        )
        .into()
}
