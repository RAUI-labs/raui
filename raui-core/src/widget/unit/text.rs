use crate::{widget::utils::Color, Scalar};
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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TextBox {
    #[serde(default)]
    pub meta: Option<String>,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub direction: TextBoxDirection,
    #[serde(default)]
    pub font: TextBoxFont,
    #[serde(default)]
    pub color: Color,
}