pub mod default_layout_engine;

use crate::widget::{unit::WidgetUnit, utils::Rect, WidgetId};
use std::collections::HashMap;

pub trait LayoutEngine<E> {
    fn layout(&mut self, ui_space: Rect, tree: &WidgetUnit) -> Result<Layout, E>;
}

#[derive(Debug, Default, Clone)]
pub struct Layout {
    pub ui_space: Rect,
    pub items: HashMap<WidgetId, LayoutItem>,
}

#[derive(Debug, Default, Clone)]
pub struct LayoutNode {
    pub id: WidgetId,
    pub local_space: Rect,
    pub children: Vec<LayoutNode>,
}

impl LayoutNode {
    pub fn count(&self) -> usize {
        1 + self.children.iter().map(Self::count).sum::<usize>()
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct LayoutItem {
    pub local_space: Rect,
    pub ui_space: Rect,
}

impl LayoutEngine<()> for () {
    fn layout(&mut self, ui_space: Rect, _: &WidgetUnit) -> Result<Layout, ()> {
        Ok(Layout {
            ui_space,
            items: Default::default(),
        })
    }
}
