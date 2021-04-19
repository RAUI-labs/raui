use crate::{
    unpack_named_slots, widget,
    widget::{context::WidgetContext, node::WidgetNode, unit::size::SizeBoxNode, utils::Rect},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct WrapBoxProps {
    #[serde(default)]
    pub margin: Rect,
}
implement_props_data!(WrapBoxProps);

pub fn wrap_box(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        props,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => content);

    let WrapBoxProps { margin } = props.read_cloned_or_default();

    widget! {{{
        SizeBoxNode {
            id: id.to_owned(),
            props: props.clone(),
            slot: Box::new(content),
            margin,
            ..Default::default()
        }
    }}}
}
