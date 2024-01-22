use crate::{
    props::Props,
    widget::{
        node::{WidgetNode, WidgetNodePrefab},
        unit::{WidgetUnit, WidgetUnitData},
        utils::{Rect, Transform, Vec2},
        WidgetId,
    },
    PrefabValue, PropsData, Scalar,
};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentBoxItemPreserveInBounds {
    #[serde(default)]
    pub width: bool,
    #[serde(default)]
    pub height: bool,
}

impl From<bool> for ContentBoxItemPreserveInBounds {
    fn from(value: bool) -> Self {
        Self {
            width: value,
            height: value,
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentBoxItemCutInBounds {
    #[serde(default)]
    pub left: bool,
    #[serde(default)]
    pub right: bool,
    #[serde(default)]
    pub top: bool,
    #[serde(default)]
    pub bottom: bool,
}

impl From<bool> for ContentBoxItemCutInBounds {
    fn from(value: bool) -> Self {
        Self {
            left: value,
            right: value,
            top: value,
            bottom: value,
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentBoxItemKeepInBounds {
    #[serde(default)]
    pub preserve: ContentBoxItemPreserveInBounds,
    #[serde(default)]
    pub cut: ContentBoxItemCutInBounds,
}

impl From<bool> for ContentBoxItemKeepInBounds {
    fn from(value: bool) -> Self {
        Self {
            preserve: value.into(),
            cut: value.into(),
        }
    }
}

/// Allows customizing how an item in a [`content_box`] is laid out
///
/// [`content_box`]: crate::widget::component::containers::content_box::content_box
#[derive(PropsData, Debug, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct ContentBoxItemLayout {
    #[serde(default = "ContentBoxItemLayout::default_anchors")]
    pub anchors: Rect,
    /// The margins to put on each side of the item
    #[serde(default)]
    pub margin: Rect,
    /// Tells in percentage, where is the center of mass of the widget, relative to it's box size
    #[serde(default)]
    pub align: Vec2,
    /// The amount to offset the item from where it would otherwise be laid out
    #[serde(default)]
    pub offset: Vec2,
    /// The "Z" depth of the item
    #[serde(default)]
    pub depth: Scalar,
    /// Set of constraints that tell if and how to keep item in container bounds
    #[serde(default)]
    pub keep_in_bounds: ContentBoxItemKeepInBounds,
}

impl ContentBoxItemLayout {
    fn default_anchors() -> Rect {
        Rect {
            left: 0.0,
            right: 1.0,
            top: 0.0,
            bottom: 1.0,
        }
    }
}

impl Default for ContentBoxItemLayout {
    fn default() -> Self {
        Self {
            anchors: Self::default_anchors(),
            margin: Default::default(),
            align: Default::default(),
            offset: Default::default(),
            depth: 0.0,
            keep_in_bounds: Default::default(),
        }
    }
}

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
pub struct ContentBox {
    #[serde(default)]
    pub id: WidgetId,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<ContentBoxItem>,
    #[serde(default)]
    pub clipping: bool,
    #[serde(default)]
    pub transform: Transform,
}

impl WidgetUnitData for ContentBox {
    fn id(&self) -> &WidgetId {
        &self.id
    }

    fn get_children(&self) -> Vec<&WidgetUnit> {
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
            transform,
            ..
        } = node;
        let items = items
            .into_iter()
            .map(ContentBoxItem::try_from)
            .collect::<Result<_, _>>()?;
        Ok(Self {
            id,
            items,
            clipping,
            transform,
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct ContentBoxNode {
    pub id: WidgetId,
    pub props: Props,
    pub items: Vec<ContentBoxItemNode>,
    pub clipping: bool,
    pub transform: Transform,
}

impl ContentBoxNode {
    pub fn remap_props<F>(&mut self, mut f: F)
    where
        F: FnMut(Props) -> Props,
    {
        let props = std::mem::take(&mut self.props);
        self.props = (f)(props);
    }
}

impl From<ContentBoxNode> for WidgetNode {
    fn from(data: ContentBoxNode) -> Self {
        Self::Unit(data.into())
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub(crate) struct ContentBoxNodePrefab {
    #[serde(default)]
    pub id: WidgetId,
    #[serde(default)]
    pub props: PrefabValue,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<ContentBoxItemNodePrefab>,
    #[serde(default)]
    pub clipping: bool,
    #[serde(default)]
    pub transform: Transform,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub(crate) struct ContentBoxItemNodePrefab {
    #[serde(default)]
    pub slot: WidgetNodePrefab,
    #[serde(default)]
    pub layout: ContentBoxItemLayout,
}
