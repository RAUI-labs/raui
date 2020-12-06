use crate::components::app::app;
use ggez::{event::EventHandler, graphics, Context, GameResult};
use raui_core::application::Application as UI;
use raui_ggez_renderer::GgezRenderer;

pub struct App {
    ui: UI,
}

impl App {
    pub fn new() -> Self {
        let mut ui = UI::new();
        ui.apply(widget! {(app)});
        Self { ui }
    }
}

impl EventHandler for App {
    fn update(&mut self, _: &mut Context) -> GameResult<()> {
        self.ui.process();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);
        self.ui
            .render(&mut GgezRenderer::new(ctx))
            .expect("GGEZ renderer could not render UI!");
        graphics::present(ctx)
    }
}
