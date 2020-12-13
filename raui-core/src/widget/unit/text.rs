use crate::{
    widget::{unit::WidgetUnitData, utils::Color, WidgetId},
    Scalar,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextBoxDirection {
    HorizontalLeftToRight,
    HorizontalRightToLeft,
    VerticalTopToBottom,
    VerticalBottomToTop,
}

impl Default for TextBoxDirection {
    fn default() -> Self {
        Self::HorizontalLeftToRight
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TextBoxFont {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub size: Scalar,
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum TextBoxSizeValue {
    Fill,
    Exact(Scalar),
}

impl Default for TextBoxSizeValue {
    fn default() -> Self {
        Self::Fill
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TextBox {
    #[serde(default)]
    pub id: WidgetId,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub width: TextBoxSizeValue,
    #[serde(default)]
    pub height: TextBoxSizeValue,
    #[serde(default)]
    pub direction: TextBoxDirection,
    #[serde(default)]
    pub font: TextBoxFont,
    #[serde(default)]
    pub color: Color,
}

impl WidgetUnitData for TextBox {
    fn id(&self) -> &WidgetId {
        &self.id
    }
}
