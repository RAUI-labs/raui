use crate::ui::components::{app::app, title_bar::title_bar};
use ggez::{event::EventHandler, graphics, Context, GameResult};
use raui_core::{application::Application as UI, prelude::*};
use raui_ggez_renderer::{GgezRenderer, GgezResources};

pub struct App {
    ui: UI,
    ui_resources: GgezResources,
}

impl App {
    pub fn new(ctx: &mut Context) -> Self {
        let mut ui_resources = GgezResources::default();
        ui_resources.fonts.insert(
            "verdana".to_owned(),
            graphics::Font::new(ctx, "/verdana.ttf").expect("GGEZ could not load `verdana.ttf`!"),
        );
        ui_resources.images.insert(
            "cat".to_owned(),
            graphics::Image::new(ctx, "/cat.jpg").expect("GGEZ could not load `cat.jpg`!"),
        );
        ui_resources.images.insert(
            "cats".to_owned(),
            graphics::Image::new(ctx, "/cats.jpg").expect("GGEZ could not load `cats.jpg`!"),
        );

        let mut ui = UI::new();
        let tree = widget! {
            (app {
                title = (title_bar)
            })
        };
        ui.apply(tree);
        Self { ui, ui_resources }
    }
}

impl EventHandler for App {
    fn update(&mut self, _: &mut Context) -> GameResult<()> {
        self.ui.process();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::WHITE);
        let (width, height) = graphics::drawable_size(ctx);
        let ui_space = Rect {
            left: 0.0,
            right: width,
            top: 0.0,
            bottom: height,
        };
        self.ui
            .layout(ui_space, &mut DefaultLayoutEngine)
            .expect("UI could not layout widgets!");
        self.ui
            .render(&mut GgezRenderer::new(ctx, &mut self.ui_resources))
            .expect("GGEZ renderer could not render UI!");
        graphics::present(ctx)
    }
}
