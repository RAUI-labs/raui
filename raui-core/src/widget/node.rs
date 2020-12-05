use crate::widget::{component::WidgetComponent, unit::WidgetUnit};

#[derive(Debug, Clone)]
pub enum WidgetNode {
    None,
    Component(WidgetComponent),
    Unit(WidgetUnit),
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

impl From<WidgetComponent> for WidgetNode {
    fn from(component: WidgetComponent) -> Self {
        Self::Component(component)
    }
}

impl From<WidgetUnit> for WidgetNode {
    fn from(unit: WidgetUnit) -> Self {
        Self::Unit(unit)
    }
}
