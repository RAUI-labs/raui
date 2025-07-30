use crate::widget::{
    WidgetId,
    node::{WidgetNode, WidgetNodePrefab},
    unit::{
        WidgetUnit, WidgetUnitData,
        content::{ContentBoxItem, ContentBoxItemNode, ContentBoxItemNodePrefab},
        flex::{FlexBoxItem, FlexBoxItemNode, FlexBoxItemNodePrefab},
        grid::{GridBoxItem, GridBoxItemNode, GridBoxItemNodePrefab},
    },
};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortalBoxSlot {
    Slot(WidgetUnit),
    ContentItem(ContentBoxItem),
    FlexItem(FlexBoxItem),
    GridItem(GridBoxItem),
}

impl Default for PortalBoxSlot {
    fn default() -> Self {
        Self::Slot(Default::default())
    }
}

impl TryFrom<PortalBoxSlotNode> for PortalBoxSlot {
    type Error = ();

    fn try_from(node: PortalBoxSlotNode) -> Result<Self, Self::Error> {
        Ok(match node {
            PortalBoxSlotNode::Slot(node) => PortalBoxSlot::Slot(WidgetUnit::try_from(node)?),
            PortalBoxSlotNode::ContentItem(item) => {
                PortalBoxSlot::ContentItem(ContentBoxItem::try_from(item)?)
            }
            PortalBoxSlotNode::FlexItem(item) => {
                PortalBoxSlot::FlexItem(FlexBoxItem::try_from(item)?)
            }
            PortalBoxSlotNode::GridItem(item) => {
                PortalBoxSlot::GridItem(GridBoxItem::try_from(item)?)
            }
        })
    }
}

#[derive(Debug, Clone)]
pub enum PortalBoxSlotNode {
    Slot(WidgetNode),
    ContentItem(ContentBoxItemNode),
    FlexItem(FlexBoxItemNode),
    GridItem(GridBoxItemNode),
}

impl Default for PortalBoxSlotNode {
    fn default() -> Self {
        Self::Slot(Default::default())
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PortalBox {
    #[serde(default)]
    pub id: WidgetId,
    #[serde(default)]
    pub slot: Box<PortalBoxSlot>,
    #[serde(default)]
    pub owner: WidgetId,
}

impl WidgetUnitData for PortalBox {
    fn id(&self) -> &WidgetId {
        &self.id
    }

    fn get_children(&self) -> Vec<&WidgetUnit> {
        vec![match &*self.slot {
            PortalBoxSlot::Slot(b) => b,
            PortalBoxSlot::ContentItem(b) => &b.slot,
            PortalBoxSlot::FlexItem(b) => &b.slot,
            PortalBoxSlot::GridItem(b) => &b.slot,
        }]
    }
}

impl TryFrom<PortalBoxNode> for PortalBox {
    type Error = ();

    fn try_from(node: PortalBoxNode) -> Result<Self, Self::Error> {
        let PortalBoxNode { id, slot, owner } = node;
        Ok(Self {
            id,
            slot: Box::new(PortalBoxSlot::try_from(*slot)?),
            owner,
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct PortalBoxNode {
    pub id: WidgetId,
    pub slot: Box<PortalBoxSlotNode>,
    pub owner: WidgetId,
}

impl From<PortalBoxNode> for WidgetNode {
    fn from(data: PortalBoxNode) -> Self {
        Self::Unit(data.into())
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub(crate) struct PortalBoxNodePrefab {
    #[serde(default)]
    pub id: WidgetId,
    #[serde(default)]
    pub slot: Box<PortalBoxSlotNodePrefab>,
    #[serde(default)]
    pub owner: WidgetId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum PortalBoxSlotNodePrefab {
    Slot(#[serde(default)] WidgetNodePrefab),
    ContentItem(#[serde(default)] ContentBoxItemNodePrefab),
    FlexItem(#[serde(default)] FlexBoxItemNodePrefab),
    GridItem(#[serde(default)] GridBoxItemNodePrefab),
}

impl Default for PortalBoxSlotNodePrefab {
    fn default() -> Self {
        Self::Slot(Default::default())
    }
}
