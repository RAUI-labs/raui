use crate::{
    widget::{
        unit::{WidgetUnit, WidgetUnitData},
        utils::Rect,
        WidgetId,
    },
    Scalar,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FlexBoxItem {
    #[serde(default)]
    pub slot: WidgetUnit,
    #[serde(default)]
    pub basis: Option<Scalar>,
    #[serde(default)]
    pub fill: Scalar,
    #[serde(default)]
    pub grow: Scalar,
    #[serde(default)]
    pub shrink: Scalar,
    #[serde(default)]
    pub margin: Rect,
    #[serde(default)]
    pub align: Scalar,
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
    pub direction: FlexBoxDirection,
    #[serde(default)]
    pub items: Vec<FlexBoxItem>,
    #[serde(default)]
    pub separation: Scalar,
    #[serde(default)]
    pub wrap: bool,
}

impl WidgetUnitData for FlexBox {
    fn id(&self) -> &WidgetId {
        &self.id
    }

    fn get_children<'a>(&'a self) -> Vec<&'a WidgetUnit> {
        self.items.iter().map(|item| &item.slot).collect()
    }
}
