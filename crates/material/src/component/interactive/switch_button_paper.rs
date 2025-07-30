use crate::component::{interactive::button_paper::button_paper, switch_paper::switch_paper};
use raui_core::{
    make_widget,
    widget::{component::WidgetComponent, context::WidgetContext, node::WidgetNode},
};

pub fn switch_button_paper(context: WidgetContext) -> WidgetNode {
    switch_button_paper_impl(make_widget!(button_paper), context)
}

pub fn switch_button_paper_impl(component: WidgetComponent, context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        idref, key, props, ..
    } = context;

    component
        .key(key)
        .maybe_idref(idref.cloned())
        .merge_props(props.clone())
        .named_slot(
            "content",
            make_widget!(switch_paper)
                .key("switch")
                .merge_props(props.clone()),
        )
        .into()
}
