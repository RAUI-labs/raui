use crate::{
    Prefab,
    props::Props,
    widget::{
        component::{WidgetComponent, WidgetComponentPrefab},
        unit::{WidgetUnitNode, WidgetUnitNodePrefab},
    },
};
use serde::{Deserialize, Serialize};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Default, Clone)]
pub enum WidgetNode {
    #[default]
    None,
    Component(WidgetComponent),
    Unit(WidgetUnitNode),
    Tuple(Vec<WidgetNode>),
}

impl WidgetNode {
    pub fn is_none(&self) -> bool {
        match self {
            Self::None => true,
            Self::Unit(unit) => unit.is_none(),
            Self::Tuple(v) => v.is_empty(),
            _ => false,
        }
    }

    pub fn is_some(&self) -> bool {
        match self {
            Self::None => false,
            Self::Unit(unit) => unit.is_some(),
            Self::Tuple(v) => !v.is_empty(),
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

    pub fn as_tuple(&self) -> Option<&[WidgetNode]> {
        match self {
            Self::Tuple(v) => Some(v),
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

    pub fn shared_props(&self) -> Option<&Props> {
        match self {
            Self::Component(c) => c.shared_props.as_ref(),
            _ => None,
        }
    }

    pub fn shared_props_mut(&mut self) -> Option<&mut Props> {
        match self {
            Self::Component(c) => {
                if c.shared_props.is_none() {
                    c.shared_props = Some(Default::default());
                }
                c.shared_props.as_mut()
            }
            _ => None,
        }
    }

    pub fn remap_shared_props<F>(&mut self, f: F)
    where
        F: FnMut(Props) -> Props,
    {
        if let Self::Component(c) = self {
            c.remap_shared_props(f);
        }
    }

    pub fn pack_tuple<const N: usize>(data: [WidgetNode; N]) -> Self {
        Self::Tuple(data.into())
    }

    pub fn unpack_tuple<const N: usize>(self) -> [WidgetNode; N] {
        if let WidgetNode::Tuple(mut data) = self {
            let mut iter = data.drain(..);
            std::array::from_fn(|_| iter.next().unwrap_or_default())
        } else {
            std::array::from_fn(|_| WidgetNode::None)
        }
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

impl<const N: usize> From<[WidgetNode; N]> for WidgetNode {
    fn from(data: [WidgetNode; N]) -> Self {
        Self::pack_tuple(data)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub(crate) enum WidgetNodePrefab {
    #[default]
    None,
    Component(WidgetComponentPrefab),
    Unit(WidgetUnitNodePrefab),
    Tuple(Vec<WidgetNodePrefab>),
}

impl Prefab for WidgetNodePrefab {}
