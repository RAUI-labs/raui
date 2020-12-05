use crate::{
    widget::{unit::WidgetUnit, utils::Rect},
    Scalar,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ListBoxItem {
    #[serde(default)]
    pub slot: WidgetUnit,
    #[serde(default)]
    pub fill: Option<Scalar>,
    #[serde(default)]
    pub margin: Rect,
    #[serde(default)]
    pub align: Scalar,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ListBoxDirection {
    HorizontalLeftToRight,
    HorizontalRightToLeft,
    VerticalTopToBottom,
    VerticalBottomToTop,
}

impl Default for ListBoxDirection {
    fn default() -> Self {
        Self::HorizontalLeftToRight
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ListBox {
    #[serde(default)]
    pub meta: Option<String>,
    #[serde(default)]
    pub direction: ListBoxDirection,
    #[serde(default)]
    pub items: Vec<ListBoxItem>,
    #[serde(default)]
    pub separation: Scalar,
}
