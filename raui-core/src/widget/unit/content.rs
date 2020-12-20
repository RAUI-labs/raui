use crate::{
    props::{Props, PropsDef},
    widget::{
        node::{WidgetNode, WidgetNodeDef},
        unit::{WidgetUnit, WidgetUnitData},
        utils::{Rect, Vec2},
        WidgetId,
    },
    Scalar,
};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ContentBoxItemLayout {
    #[serde(default)]
    pub anchors: Rect,
    #[serde(default)]
    pub margin: Rect,
    #[serde(default)]
    pub align: Vec2,
    #[serde(default)]
    pub offset: Vec2,
    #[serde(default)]
    pub depth: Scalar,
}
implement_props_data!(ContentBoxItemLayout, "ContentBoxItemLayout");

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ContentBoxItem {
    #[serde(default)]
    pub slot: WidgetUnit,
    #[serde(default)]
    pub layout: ContentBoxItemLayout,
}

impl TryFrom<ContentBoxItemNode> for ContentBoxItem {
    type Error = ();

    fn try_from(node: ContentBoxItemNode) -> Result<Self, Self::Error> {
        let ContentBoxItemNode { slot, layout } = node;
        Ok(Self {
            slot: WidgetUnit::try_from(slot)?,
            layout,
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct ContentBoxItemNode {
    pub slot: WidgetNode,
    pub layout: ContentBoxItemLayout,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ContentBoxItemNodeDef {
    #[serde(default)]
    pub slot: WidgetNodeDef,
    #[serde(default)]
    pub layout: ContentBoxItemLayout,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ContentBox {
    #[serde(default)]
    pub id: WidgetId,
    #[serde(default)]
    pub items: Vec<ContentBoxItem>,
    #[serde(default)]
    pub clipping: bool,
}

impl WidgetUnitData for ContentBox {
    fn id(&self) -> &WidgetId {
        &self.id
    }

    fn get_children<'a>(&'a self) -> Vec<&'a WidgetUnit> {
        self.items.iter().map(|item| &item.slot).collect()
    }
}

impl TryFrom<ContentBoxNode> for ContentBox {
    type Error = ();

    fn try_from(node: ContentBoxNode) -> Result<Self, Self::Error> {
        let ContentBoxNode {
            id,
            items,
            clipping,
            ..
        } = node;
        let items = items
            .into_iter()
            .map(|item| ContentBoxItem::try_from(item))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            id,
            items,
            clipping,
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct ContentBoxNode {
    pub id: WidgetId,
    pub props: Props,
    pub items: Vec<ContentBoxItemNode>,
    pub clipping: bool,
}

impl ContentBoxNode {
    pub fn remap_props<F>(&mut self, mut f: F)
    where
        F: FnMut(Props) -> Props,
    {
        let props = std::mem::replace(&mut self.props, Default::default());
        self.props = (f)(props);
    }
}

impl Into<WidgetNode> for ContentBoxNode {
    fn into(self) -> WidgetNode {
        WidgetNode::Unit(self.into())
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ContentBoxNodeDef {
    #[serde(default)]
    pub id: WidgetId,
    #[serde(default)]
    pub props: PropsDef,
    #[serde(default)]
    pub items: Vec<ContentBoxItemNodeDef>,
    #[serde(default)]
    pub clipping: bool,
}
