pub mod content;
pub mod image;
pub mod list;
pub mod text;

use crate::widget::{
    node::WidgetNode,
    unit::{content::ContentBox, image::ImageBox, list::ListBox, text::TextBox},
};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetUnit {
    None,
    ContentBox(ContentBox),
    ImageBox(ImageBox),
    TextBox(TextBox),
    ListBox(ListBox),
}

impl Default for WidgetUnit {
    fn default() -> Self {
        Self::None
    }
}

impl WidgetUnit {
    pub fn is_none(&self) -> bool {
        match self {
            Self::None => true,
            _ => false,
        }
    }

    pub fn is_some(&self) -> bool {
        match self {
            Self::None => false,
            _ => true,
        }
    }
}

impl TryFrom<WidgetNode> for WidgetUnit {
    type Error = ();

    fn try_from(node: WidgetNode) -> Result<Self, Self::Error> {
        if let WidgetNode::Unit(v) = node {
            Ok(v)
        } else {
            Err(())
        }
    }
}

impl From<()> for WidgetUnit {
    fn from(_: ()) -> Self {
        Self::None
    }
}

macro_rules! implement_from_unit {
    { $( $type_name:ident ),+ } => {
        $(
            impl From<$type_name> for WidgetUnit {
                fn from(unit: $type_name) -> Self {
                    Self::$type_name(unit)
                }
            }
        )+
    };
}

implement_from_unit! {
    ContentBox,
    ImageBox,
    ListBox,
    TextBox
}
