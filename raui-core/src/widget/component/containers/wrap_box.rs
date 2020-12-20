use crate::{
    unpack_named_slots, widget,
    widget::{unit::size::SizeBoxNode, utils::Rect},
    widget_component,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct WrapBoxProps {
    #[serde(default)]
    pub margin: Rect,
}
implement_props_data!(WrapBoxProps, "WrapBoxProps");

widget_component! {
    pub wrap_box(id, props, named_slots) {
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
}
