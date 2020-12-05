use crate::widget::unit::WidgetUnit;

pub trait Renderer<T, E> {
    fn render(&mut self, tree: &WidgetUnit) -> Result<T, E>;
}

#[derive(Debug, Default, Copy, Clone)]
pub struct RawRenderer;

impl Renderer<WidgetUnit, ()> for RawRenderer {
    fn render(&mut self, tree: &WidgetUnit) -> Result<WidgetUnit, ()> {
        Ok(tree.clone())
    }
}
