use crate::component::{interactive::button_paper::button_paper, text_paper::text_paper};
use raui_core::{
    make_widget,
    widget::{
        component::{
            WidgetComponent,
            containers::wrap_box::{WrapBoxProps, wrap_box},
        },
        context::WidgetContext,
        node::WidgetNode,
    },
};

pub fn text_button_paper(context: WidgetContext) -> WidgetNode {
    text_button_paper_impl(make_widget!(button_paper), context)
}

pub fn text_button_paper_impl(component: WidgetComponent, context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        idref, key, props, ..
    } = context;

    let wrap_props = props.read_cloned_or_default::<WrapBoxProps>();

    component
        .key(key)
        .maybe_idref(idref.cloned())
        .merge_props(props.clone())
        .named_slot(
            "content",
            make_widget!(wrap_box)
                .key("wrap")
                .with_props(wrap_props)
                .named_slot(
                    "content",
                    make_widget!(text_paper)
                        .key("switch")
                        .merge_props(props.clone()),
                ),
        )
        .into()
}
