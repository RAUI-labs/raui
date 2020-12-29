use crate::{
    props::Props,
    widget::{
        component::{WidgetComponent, WidgetComponentDef},
        unit::{WidgetUnitNode, WidgetUnitNodeDef},
    },
};
use serde::{Deserialize, Serialize};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum WidgetNode {
    None,
    Component(WidgetComponent),
    Unit(WidgetUnitNode),
}

impl WidgetNode {
    pub fn is_none(&self) -> bool {
        match self {
            Self::None => true,
            Self::Unit(unit) => unit.is_none(),
            _ => false,
        }
    }

    pub fn is_some(&self) -> bool {
        match self {
            Self::None => false,
            Self::Unit(unit) => unit.is_some(),
            _ => true,
        }
    }

    pub fn as_component(&self) -> Option<&WidgetComponent> {
        match self {
            Self::Component(c) => Some(c),
            _ => None,
        }
    }

    pub fn as_unit(&self) -> Option<&WidgetUnitNode> {
        match self {
            Self::Unit(u) => Some(u),
            _ => None,
        }
    }

    pub fn props(&self) -> Option<&Props> {
        match self {
            Self::Component(c) => Some(&c.props),
            Self::Unit(u) => u.props(),
            _ => None,
        }
    }

    pub fn props_mut(&mut self) -> Option<&mut Props> {
        match self {
            Self::Component(c) => Some(&mut c.props),
            Self::Unit(u) => u.props_mut(),
            _ => None,
        }
    }

    pub fn remap_props<F>(&mut self, f: F)
    where
        F: FnMut(Props) -> Props,
    {
        match self {
            Self::Component(c) => c.remap_props(f),
            Self::Unit(u) => u.remap_props(f),
            _ => {}
        }
    }
}

impl Default for WidgetNode {
    fn default() -> Self {
        Self::None
    }
}

impl From<()> for WidgetNode {
    fn from(_: ()) -> Self {
        Self::None
    }
}

impl From<()> for Box<WidgetNode> {
    fn from(_: ()) -> Self {
        Box::new(WidgetNode::None)
    }
}

impl From<WidgetComponent> for WidgetNode {
    fn from(component: WidgetComponent) -> Self {
        Self::Component(component)
    }
}

impl From<WidgetUnitNode> for WidgetNode {
    fn from(unit: WidgetUnitNode) -> Self {
        Self::Unit(unit)
    }
}

impl From<WidgetUnitNode> for Box<WidgetNode> {
    fn from(unit: WidgetUnitNode) -> Self {
        Box::new(WidgetNode::Unit(unit))
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetNodeDef {
    None,
    Component(WidgetComponentDef),
    Unit(WidgetUnitNodeDef),
}

impl Default for WidgetNodeDef {
    fn default() -> Self {
        Self::None
    }
}
