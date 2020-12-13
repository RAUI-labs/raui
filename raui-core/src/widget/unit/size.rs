use crate::{
    widget::{
        unit::{WidgetUnit, WidgetUnitData},
        utils::Rect,
        WidgetId,
    },
    Scalar,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum SizeBoxSizeValue {
    Content,
    Fill,
    Exact(Scalar),
}

impl Default for SizeBoxSizeValue {
    fn default() -> Self {
        Self::Content
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SizeBox {
    #[serde(default)]
    pub id: WidgetId,
    #[serde(default)]
    pub slot: Box<WidgetUnit>,
    #[serde(default)]
    pub width: SizeBoxSizeValue,
    #[serde(default)]
    pub height: SizeBoxSizeValue,
    #[serde(default)]
    pub margin: Rect,
}

impl WidgetUnitData for SizeBox {
    fn id(&self) -> &WidgetId {
        &self.id
    }

    fn get_children<'a>(&'a self) -> Vec<&'a WidgetUnit> {
        vec![&self.slot]
    }
}
