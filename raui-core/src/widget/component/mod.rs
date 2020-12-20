pub mod containers;
pub mod image_box;
pub mod interactive;
pub mod space_box;
pub mod text_box;

use crate::{
    application::Application,
    props::{Props, PropsDef},
    widget::{
        node::{WidgetNode, WidgetNodeDef},
        FnWidget,
    },
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::TryFrom};

#[derive(Clone)]
pub struct WidgetComponent {
    pub processor: FnWidget,
    pub type_name: String,
    pub key: Option<String>,
    pub props: Props,
    pub listed_slots: Vec<WidgetNode>,
    pub named_slots: HashMap<String, WidgetNode>,
}

impl WidgetComponent {
    pub fn remap_props<F>(&mut self, mut f: F)
    where
        F: FnMut(Props) -> Props,
    {
        let props = std::mem::replace(&mut self.props, Default::default());
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetComponentDef {
    #[serde(default)]
    pub type_name: String,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub props: PropsDef,
    #[serde(default)]
    pub listed_slots: Vec<WidgetNodeDef>,
    #[serde(default)]
    pub named_slots: HashMap<String, WidgetNodeDef>,
}

pub fn install_components(app: &mut Application) {
    app.map_component("content_box", containers::content_box::content_box);
    app.map_component("flex_box", containers::flex_box::flex_box);
    app.map_component("grid_box", containers::grid_box::grid_box);
    app.map_component("horizontal_box", containers::horizontal_box::horizontal_box);
    app.map_component("size_box", containers::size_box::size_box);
    app.map_component("switch_box", containers::switch_box::switch_box);
    app.map_component("variant_box", containers::variant_box::variant_box);
    app.map_component("vertical_box", containers::vertical_box::vertical_box);
    app.map_component("wrap_box", containers::wrap_box::wrap_box);
    app.map_component("image_box", image_box::image_box);
    app.map_component("space_box", space_box::space_box);
    app.map_component("text_box", text_box::text_box);
}
