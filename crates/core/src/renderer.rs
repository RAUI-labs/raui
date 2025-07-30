//! Renderer traits

use crate::{
    layout::{CoordsMapping, Layout},
    widget::unit::WidgetUnit,
};

pub trait Renderer<T, E> {
    fn render(
        &mut self,
        tree: &WidgetUnit,
        mapping: &CoordsMapping,
        layout: &Layout,
    ) -> Result<T, E>;
}

#[derive(Debug, Default, Copy, Clone)]
pub struct RawRenderer;

impl Renderer<WidgetUnit, ()> for RawRenderer {
    fn render(
        &mut self,
        tree: &WidgetUnit,
        _: &CoordsMapping,
        _: &Layout,
    ) -> Result<WidgetUnit, ()> {
        Ok(tree.clone())
    }
}
