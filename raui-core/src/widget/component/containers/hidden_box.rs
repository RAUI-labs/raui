use crate::{
    unpack_named_slots, widget,
    widget::{context::WidgetContext, node::WidgetNode, unit::area::AreaBoxNode},
    PropsData,
};
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Copy, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct HiddenBoxProps(#[serde(default)] pub bool);

pub fn hidden_box(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        props,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => content);

    let HiddenBoxProps(hidden) = props.read_cloned_or_default();

    if hidden {
        widget! {()}
    } else {
        widget! {{{
            AreaBoxNode {
                id: id.to_owned(),
                slot: Box::new(content),
                renderer_effect: None,
            }
        }}}
    }
}
