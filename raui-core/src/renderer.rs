use crate::{layout::Layout, widget::unit::WidgetUnit};

pub trait Renderer<T, E> {
    fn render(&mut self, tree: &WidgetUnit, layout: &Layout) -> Result<T, E>;
}

#[derive(Debug, Default, Copy, Clone)]
pub struct RawRenderer;

impl Renderer<WidgetUnit, ()> for RawRenderer {
    fn render(&mut self, tree: &WidgetUnit, _: &Layout) -> Result<WidgetUnit, ()> {
        Ok(tree.clone())
    }
}
