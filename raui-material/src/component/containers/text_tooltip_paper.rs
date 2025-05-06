use crate::component::{containers::tooltip_paper::tooltip_paper, text_paper::text_paper};
use raui_core::{
    make_widget, unpack_named_slots,
    widget::{context::WidgetContext, node::WidgetNode},
};

pub fn text_tooltip_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        idref,
        key,
        props,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => content);

    make_widget!(tooltip_paper)
        .key(key)
        .maybe_idref(idref.cloned())
        .merge_props(props.clone())
        .named_slot("content", content)
        .named_slot(
            "tooltip",
            make_widget!(text_paper)
                .key("text")
                .merge_props(props.clone()),
        )
        .into()
}
