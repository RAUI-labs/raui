use crate::{
    props::Props,
    widget::{
        node::{WidgetNode, WidgetNodePrefab},
        unit::{WidgetUnit, WidgetUnitData},
        utils::{Rect, Transform},
        WidgetId,
    },
    PrefabValue, Scalar,
};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlexBoxItemLayout {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub basis: Option<Scalar>,
    #[serde(default = "FlexBoxItemLayout::default_fill")]
    pub fill: Scalar,
    #[serde(default = "FlexBoxItemLayout::default_grow")]
    pub grow: Scalar,
    #[serde(default = "FlexBoxItemLayout::default_shrink")]
    pub shrink: Scalar,
    #[serde(default)]
    pub margin: Rect,
    #[serde(default)]
    pub align: Scalar,
}
implement_props_data!(FlexBoxItemLayout);

impl FlexBoxItemLayout {
    fn default_fill() -> Scalar {
        1.0
    }

    fn default_grow() -> Scalar {
        1.0
    }

    fn default_shrink() -> Scalar {
        1.0
    }
}

impl Default for FlexBoxItemLayout {
    fn default() -> Self {
        Self {
            basis: None,
            fill: Self::default_fill(),
            grow: Self::default_grow(),
            shrink: Self::default_shrink(),
            margin: Default::default(),
            align: 0.0,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FlexBoxItem {
    #[serde(default)]
    pub slot: WidgetUnit,
    #[serde(default)]
    pub layout: FlexBoxItemLayout,
}

impl TryFrom<FlexBoxItemNode> for FlexBoxItem {
    type Error = ();

    fn try_from(node: FlexBoxItemNode) -> Result<Self, Self::Error> {
        let FlexBoxItemNode { slot, layout } = node;
        Ok(Self {
            slot: WidgetUnit::try_from(slot)?,
            layout,
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct FlexBoxItemNode {
    pub slot: WidgetNode,
    pub layout: FlexBoxItemLayout,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlexBoxDirection {
    HorizontalLeftToRight,
    HorizontalRightToLeft,
    VerticalTopToBottom,
    VerticalBottomToTop,
}

impl Default for FlexBoxDirection {
    fn default() -> Self {
        Self::HorizontalLeftToRight
    }
}

impl FlexBoxDirection {
    pub fn is_horizontal(&self) -> bool {
        *self == Self::HorizontalLeftToRight || *self == Self::HorizontalRightToLeft
    }

    pub fn is_vertical(&self) -> bool {
        *self == Self::VerticalTopToBottom || *self == Self::VerticalBottomToTop
    }

    pub fn is_order_ascending(&self) -> bool {
        *self == Self::HorizontalLeftToRight || *self == Self::VerticalTopToBottom
    }

    pub fn is_order_descending(&self) -> bool {
        *self == Self::HorizontalRightToLeft || *self == Self::VerticalBottomToTop
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FlexBox {
    #[serde(default)]
    pub id: WidgetId,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<FlexBoxItem>,
    #[serde(default)]
    pub direction: FlexBoxDirection,
    #[serde(default)]
    pub separation: Scalar,
    #[serde(default)]
    pub wrap: bool,
    #[serde(default)]
    pub transform: Transform,
}

impl WidgetUnitData for FlexBox {
    fn id(&self) -> &WidgetId {
        &self.id
    }

    fn get_children(&self) -> Vec<&WidgetUnit> {
        self.items.iter().map(|item| &item.slot).collect()
    }
}

impl TryFrom<FlexBoxNode> for FlexBox {
    type Error = ();

    fn try_from(node: FlexBoxNode) -> Result<Self, Self::Error> {
        let FlexBoxNode {
            id,
            items,
            direction,
            separation,
            wrap,
            transform,
            ..
        } = node;
        let items = items
            .into_iter()
            .map(FlexBoxItem::try_from)
            .collect::<Result<_, _>>()?;
        Ok(Self {
            id,
            items,
            direction,
            separation,
            wrap,
            transform,
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct FlexBoxNode {
    pub id: WidgetId,
    pub props: Props,
    pub items: Vec<FlexBoxItemNode>,
    pub direction: FlexBoxDirection,
    pub separation: Scalar,
    pub wrap: bool,
    pub transform: Transform,
}

impl FlexBoxNode {
    pub fn remap_props<F>(&mut self, mut f: F)
    where
        F: FnMut(Props) -> Props,
    {
        let props = std::mem::take(&mut self.props);
        self.props = (f)(props);
    }
}

impl Into<WidgetNode> for FlexBoxNode {
    fn into(self) -> WidgetNode {
        WidgetNode::Unit(self.into())
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub(crate) struct FlexBoxNodePrefab {
    #[serde(default)]
    pub id: WidgetId,
    #[serde(default)]
    pub props: PrefabValue,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<FlexBoxItemNodePrefab>,
    #[serde(default)]
    pub direction: FlexBoxDirection,
    #[serde(default)]
    pub separation: Scalar,
    #[serde(default)]
    pub wrap: bool,
    #[serde(default)]
    pub transform: Transform,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub(crate) struct FlexBoxItemNodePrefab {
    #[serde(default)]
    pub slot: WidgetNodePrefab,
    #[serde(default)]
    pub layout: FlexBoxItemLayout,
}
