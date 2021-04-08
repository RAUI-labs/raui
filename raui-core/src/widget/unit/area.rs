use crate::{Scalar, widget::{
    node::{WidgetNode, WidgetNodePrefab},
    unit::{WidgetUnit, WidgetUnitData},
    WidgetId,
}};
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AreaBoxRendererEffect {
    pub id: String,
    pub params: [Scalar; 8],
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AreaBox {
    #[serde(default)]
    pub id: WidgetId,
    #[serde(default)]
    pub slot: Box<WidgetUnit>,
    #[serde(default)]
    pub renderer_effect: Option<AreaBoxRendererEffect>,
}

impl WidgetUnitData for AreaBox {
    fn id(&self) -> &WidgetId {
        &self.id
    }

    fn get_children(&self) -> Vec<&WidgetUnit> {
        vec![&self.slot]
    }
}

impl TryFrom<AreaBoxNode> for AreaBox {
    type Error = ();

    fn try_from(node: AreaBoxNode) -> Result<Self, Self::Error> {
        let AreaBoxNode {
            id,
            slot,
            renderer_effect,
        } = node;
        Ok(Self {
            id,
            slot: Box::new(WidgetUnit::try_from(*slot)?),
            renderer_effect,
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct AreaBoxNode {
    pub id: WidgetId,
    pub slot: Box<WidgetNode>,
    pub renderer_effect: Option<AreaBoxRendererEffect>,
}

impl From<AreaBoxNode> for WidgetNode {
    fn from(data: AreaBoxNode) -> Self {
        Self::Unit(data.into())
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub(crate) struct AreaBoxNodePrefab {
    #[serde(default)]
    pub id: WidgetId,
    #[serde(default)]
    pub slot: Box<WidgetNodePrefab>,
    #[serde(default)]
    pub renderer_effect: Option<AreaBoxRendererEffect>,
}
