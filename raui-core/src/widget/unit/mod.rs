pub mod content;
pub mod flex;
pub mod grid;
pub mod image;
pub mod size;
pub mod text;

use crate::widget::{
    node::WidgetNode,
    unit::{
        content::ContentBox, flex::FlexBox, grid::GridBox, image::ImageBox, size::SizeBox,
        text::TextBox,
    },
    WidgetId,
};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct WidgetUnitInspectionNode {
    pub id: WidgetId,
    pub children: Vec<WidgetUnitInspectionNode>,
}

pub trait WidgetUnitData {
    fn id(&self) -> &WidgetId;

    fn get_children<'a>(&'a self) -> Vec<&'a WidgetUnit> {
        vec![]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetUnit {
    None,
    ContentBox(ContentBox),
    FlexBox(FlexBox),
    GridBox(GridBox),
    SizeBox(SizeBox),
    ImageBox(ImageBox),
    TextBox(TextBox),
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

    pub fn as_data(&self) -> Option<&dyn WidgetUnitData> {
        match self {
            Self::None => None,
            Self::ContentBox(v) => Some(v as &dyn WidgetUnitData),
            Self::FlexBox(v) => Some(v as &dyn WidgetUnitData),
            Self::GridBox(v) => Some(v as &dyn WidgetUnitData),
            Self::SizeBox(v) => Some(v as &dyn WidgetUnitData),
            Self::ImageBox(v) => Some(v as &dyn WidgetUnitData),
            Self::TextBox(v) => Some(v as &dyn WidgetUnitData),
        }
    }

    pub fn inspect(&self) -> Option<WidgetUnitInspectionNode> {
        if let Some(data) = self.as_data() {
            Some(WidgetUnitInspectionNode {
                id: data.id().to_owned(),
                children: data
                    .get_children()
                    .into_iter()
                    .filter_map(|child| child.inspect())
                    .collect::<Vec<_>>(),
            })
        } else {
            None
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
    { $( $type_name:ident ),+ $(,)? } => {
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
    FlexBox,
    GridBox,
    SizeBox,
    ImageBox,
    TextBox,
}
