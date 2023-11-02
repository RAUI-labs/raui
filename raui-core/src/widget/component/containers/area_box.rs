use crate::{
    unpack_named_slots,
    widget::{context::WidgetContext, node::WidgetNode, unit::area::AreaBoxNode},
};

pub fn area_box(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id, named_slots, ..
    } = context;
    unpack_named_slots!(named_slots => content);

    AreaBoxNode {
        id: id.to_owned(),
        slot: Box::new(content),
    }
    .into()
}
