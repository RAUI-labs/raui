use crate::{
    widget,
    widget::unit::grid::{GridBoxItemLayout, GridBoxItemNode, GridBoxNode},
    widget_component,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GridBoxProps {
    #[serde(default)]
    pub cols: usize,
    #[serde(default)]
    pub rows: usize,
}
implement_props_data!(GridBoxProps, "GridBoxProps");

widget_component! {
    pub grid_box(id, props, listed_slots) {
        let items = listed_slots.into_iter().map(|slot| {
            let layout = slot
                .props()
                .expect("WidgetNode has no Props")
                .read_cloned_or_default::<GridBoxItemLayout>();
            GridBoxItemNode {
                slot,
                layout,
            }
        }).collect::<Vec<_>>();
        let GridBoxProps { cols, rows } = props.read_cloned_or_default();

        widget! {{{
            GridBoxNode {
                id: id.to_owned(),
                props: props.clone(),
                items,
                cols,
                rows,
            }
        }}}
    }
}
