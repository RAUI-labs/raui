use crate::ui::components::item_cell::{
    item_cell, ItemCellProps, ItemCellsProps, ItemData, OwningInventoryProps,
};
use raui_core::prelude::*;
use raui_material::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct InventoryState {
    pub index: usize,
    pub count: usize,
}

impl Default for InventoryState {
    fn default() -> Self {
        Self { index: 0, count: 3 }
    }
}

#[derive(MessageData, Debug, Copy, Clone)]
pub enum InventoryMessage {
    Prev,
    Next,
}

fn use_inventory(context: &mut WidgetContext) {
    context.life_cycle.change(|context| {
        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<InventoryMessage>() {
                match msg {
                    InventoryMessage::Prev => {
                        let mut data = match context.state.read::<InventoryState>() {
                            Ok(state) => *state,
                            Err(_) => InventoryState::default(),
                        };
                        data.index = data.index.saturating_sub(1);
                        drop(context.state.write(data));
                    }
                    InventoryMessage::Next => {
                        let mut data = match context.state.read::<InventoryState>() {
                            Ok(state) => *state,
                            Err(_) => InventoryState::default(),
                        };
                        let count = context
                            .props
                            .map_or_default::<ItemCellsProps, _, _>(|p| p.items.len());
                        data.index = (data.index + 1).min(count.saturating_sub(data.count));
                        drop(context.state.write(data));
                    }
                }
            }
        }
    });
}

#[pre_hooks(use_inventory)]
pub fn inventory(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        key,
        props,
        state,
        ..
    } = context;

    let ItemCellsProps { items } = props.read_cloned_or_default();
    let data = match state.read::<InventoryState>() {
        Ok(data) => *data,
        Err(_) => InventoryState::default(),
    };
    let list_props = Props::new(PaperProps {
        frame: None,
        ..Default::default()
    })
    .with(ContentBoxItemLayout {
        margin: Rect {
            left: 5.0,
            right: 5.0,
            top: 4.0,
            bottom: 4.0,
        },
        ..Default::default()
    });
    let shared_props = OwningInventoryProps(id.to_owned());
    let mut children = Vec::with_capacity(2 + data.count);
    children.push({
        let item_props = Props::new(FlexBoxItemLayout {
            grow: 0.0,
            shrink: 0.0,
            ..Default::default()
        })
        .with(ItemCellProps {
            image: "icon-prev".to_owned(),
            thin: true,
        });
        widget! {
            (#{"prev"} item_cell: {item_props})
        }
    });
    for i in 0..(data.count) {
        let item = items.get(data.index + i).cloned().unwrap_or_default();
        let item_props = Props::new(FlexBoxItemLayout {
            grow: 0.0,
            shrink: 0.0,
            ..Default::default()
        })
        .with(ItemData {
            index: data.index + i,
        })
        .with(item);
        children.push(widget! {
            (#{i} item_cell: {item_props})
        });
    }
    children.push({
        let item_props = Props::new(FlexBoxItemLayout {
            grow: 0.0,
            shrink: 0.0,
            ..Default::default()
        })
        .with(ItemCellProps {
            image: "icon-next".to_owned(),
            thin: true,
        });
        widget! {
            (#{"next"} item_cell: {item_props})
        }
    });

    widget! {
        (#{key} size_box {
            content = (#{"cells"} flex_paper: {list_props} | {shared_props} |[ children ]|)
        })
    }
}
