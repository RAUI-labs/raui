use crate::{
    widget::{
        unit::WidgetUnit,
        utils::{Rect, Vec2},
    },
    Scalar,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ContentBoxItemLayout {
    #[serde(default)]
    pub anchors: Rect,
    #[serde(default)]
    pub margin: Rect,
    #[serde(default)]
    pub pivot: Vec2,
    #[serde(default)]
    pub offset: Vec2,
    #[serde(default)]
    pub depth: Scalar,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ContentBoxItem {
    #[serde(default)]
    pub slot: WidgetUnit,
    #[serde(default)]
    pub layout: ContentBoxItemLayout,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ContentBox {
    #[serde(default)]
    pub meta: Option<String>,
    #[serde(default)]
    pub items: Vec<ContentBoxItem>,
    #[serde(default)]
    pub clipping: bool,
    #[serde(default)]
    pub size_to_content: bool,
}
