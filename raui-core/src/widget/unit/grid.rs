use crate::{
    props::{Props, PropsDef},
    widget::{
        node::{WidgetNode, WidgetNodeDef},
        unit::{WidgetUnit, WidgetUnitData},
        utils::{IntRect, Rect},
        WidgetId,
    },
    Scalar,
};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GridBoxItemLayout {
    #[serde(default)]
    pub space_occupancy: IntRect,
    #[serde(default)]
    pub margin: Rect,
    #[serde(default)]
    pub horizontal_align: Scalar,
    #[serde(default)]
    pub vertical_align: Scalar,
}
implement_props_data!(GridBoxItemLayout, "GridBoxItemLayout");

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GridBoxItem {
    #[serde(default)]
    pub slot: WidgetUnit,
    #[serde(default)]
    pub layout: GridBoxItemLayout,
}

impl TryFrom<GridBoxItemNode> for GridBoxItem {
    type Error = ();

    fn try_from(node: GridBoxItemNode) -> Result<Self, Self::Error> {
        let GridBoxItemNode { slot, layout } = node;
        Ok(Self {
            slot: WidgetUnit::try_from(slot)?,
            layout,
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct GridBoxItemNode {
    pub slot: WidgetNode,
    pub layout: GridBoxItemLayout,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GridBoxItemNodeDef {
    #[serde(default)]
    pub slot: WidgetNodeDef,
    #[serde(default)]
    pub layout: GridBoxItemLayout,
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

impl TryFrom<GridBoxNode> for GridBox {
    type Error = ();

    fn try_from(node: GridBoxNode) -> Result<Self, Self::Error> {
        let GridBoxNode {
            id,
            items,
            cols,
            rows,
            ..
        } = node;
        let items = items
            .into_iter()
            .map(GridBoxItem::try_from)
            .collect::<Result<_, _>>()?;
        Ok(Self {
            id,
            items,
            cols,
            rows,
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct GridBoxNode {
    pub id: WidgetId,
    pub props: Props,
    pub items: Vec<GridBoxItemNode>,
    pub cols: usize,
    pub rows: usize,
}

impl GridBoxNode {
    pub fn remap_props<F>(&mut self, mut f: F)
    where
        F: FnMut(Props) -> Props,
    {
        let props = std::mem::take(&mut self.props);
        self.props = (f)(props);
    }
}

impl Into<WidgetNode> for GridBoxNode {
    fn into(self) -> WidgetNode {
        WidgetNode::Unit(self.into())
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GridBoxNodeDef {
    #[serde(default)]
    pub id: WidgetId,
    #[serde(default)]
    pub props: PropsDef,
    #[serde(default)]
    pub items: Vec<GridBoxItemNodeDef>,
    #[serde(default)]
    pub cols: usize,
    #[serde(default)]
    pub rows: usize,
}
