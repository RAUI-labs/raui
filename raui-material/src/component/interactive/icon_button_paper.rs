use crate::component::{icon_paper::icon_paper, interactive::button_paper::button_paper};
use raui_core::prelude::*;

pub fn icon_button_paper(context: WidgetContext) -> WidgetNode {
    icon_button_paper_impl(make_widget!(button_paper), context)
}

pub fn icon_button_paper_impl(component: WidgetComponent, context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        idref, key, props, ..
    } = context;

    component
        .key(key)
        .maybe_idref(idref.cloned())
        .merge_props(props.clone())
        .named_slot(
            "content",
            make_widget!(icon_paper)
                .key("icon")
                .merge_props(props.clone()),
        )
        .into()
}
