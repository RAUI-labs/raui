use crate::{
    widget::{
        unit::{WidgetUnit, WidgetUnitData},
        utils::{IntRect, Rect},
        WidgetId,
    },
    Scalar,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GridBoxItem {
    #[serde(default)]
    pub slot: WidgetUnit,
    #[serde(default)]
    pub space_occupancy: IntRect,
    #[serde(default)]
    pub margin: Rect,
    #[serde(default)]
    pub horizontal_align: Scalar,
    pub vertical_align: Scalar,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GridBox {
    #[serde(default)]
    pub id: WidgetId,
    #[serde(default)]
    pub items: Vec<GridBoxItem>,
    #[serde(default)]
    pub cols: usize,
    #[serde(default)]
    pub rows: usize,
}

impl WidgetUnitData for GridBox {
    fn id(&self) -> &WidgetId {
        &self.id
    }

    fn get_children<'a>(&'a self) -> Vec<&'a WidgetUnit> {
        self.items.iter().map(|item| &item.slot).collect()
    }
}
