pub mod containers;
pub mod image_box;
pub mod interactive;
pub mod space_box;
pub mod text_box;

use crate::{
    props::Props,
    widget::{
        node::{WidgetNode, WidgetNodePrefab},
        FnWidget,
    },
    PrefabValue, Scalar,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::TryFrom};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct WidgetAlpha(pub Scalar);
implement_props_data!(WidgetAlpha);

impl Default for WidgetAlpha {
    fn default() -> Self {
        Self(1.0)
    }
}

impl WidgetAlpha {
    pub fn multiply(&mut self, alpha: Scalar) {
        self.0 *= alpha;
    }
}

#[derive(Clone)]
pub struct WidgetComponent {
    pub processor: FnWidget,
    pub type_name: String,
    pub key: Option<String>,
    pub props: Props,
    pub shared_props: Option<Props>,
    pub listed_slots: Vec<WidgetNode>,
    pub named_slots: HashMap<String, WidgetNode>,
}

impl WidgetComponent {
    pub fn remap_props<F>(&mut self, mut f: F)
    where
        F: FnMut(Props) -> Props,
    {
        let props = std::mem::take(&mut self.props);
        self.props = (f)(props);
    }
}

impl std::fmt::Debug for WidgetComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("WidgetComponent");
        s.field("type_name", &self.type_name);
        if let Some(key) = &self.key {
            s.field("key", key);
        }
        s.field("props", &self.props);
        s.field("shared_props", &self.shared_props);
        if !self.listed_slots.is_empty() {
            s.field("listed_slots", &self.listed_slots);
        }
        if !self.named_slots.is_empty() {
            s.field("named_slots", &self.named_slots);
        }
        s.finish()
    }
}

impl TryFrom<WidgetNode> for WidgetComponent {
    type Error = ();

    fn try_from(node: WidgetNode) -> Result<Self, Self::Error> {
        if let WidgetNode::Component(v) = node {
            Ok(v)
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub(crate) struct WidgetComponentPrefab {
    #[serde(default)]
    pub type_name: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(default)]
    pub props: PrefabValue,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared_props: Option<PrefabValue>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub listed_slots: Vec<WidgetNodePrefab>,
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub named_slots: HashMap<String, WidgetNodePrefab>,
}
