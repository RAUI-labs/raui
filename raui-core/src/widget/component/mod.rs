use crate::{
    props::Props,
    widget::{node::WidgetNode, FnWidget},
};
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

impl std::fmt::Debug for WidgetComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("WidgetComponent");
        s.field("type_name", &self.type_name);
        if let Some(key) = &self.key {
            s.field("key", key);
        }
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
