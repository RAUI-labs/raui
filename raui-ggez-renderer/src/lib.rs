use ggez::{graphics, Context};
use raui_core::{layout::Layout, renderer::Renderer, widget::unit::WidgetUnit};

pub struct GgezRenderer<'a> {
    context: &'a mut Context,
}

impl<'a> GgezRenderer<'a> {
    pub fn new(context: &'a mut Context) -> Self {
        Self { context }
    }
}

impl<'a> Renderer<(), ()> for GgezRenderer<'a> {
    fn render(&mut self, _tree: &WidgetUnit, _layout: &Layout) -> Result<(), ()> {
        let _size = graphics::size(self.context);
        Ok(())
    }
}
