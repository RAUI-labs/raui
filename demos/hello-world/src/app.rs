use crate::ui::components::{app::app, content::content, title_bar::title_bar};
use ggez::{
    event::EventHandler,
    graphics,
    input::keyboard::{KeyCode, KeyMods},
    Context, GameResult,
};
use raui_core::{application::Application as UI, prelude::*};
use raui_ggez_renderer::prelude::*;

pub struct App {
    ui: UI,
    ui_interactions: GgezInteractionsEngine,
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
        ui.setup(install_components);
        let tree = widget! {
            (app {
                title = (title_bar)
                content = (content)
            })
        };
        ui.apply(tree);
        let ui_interactions = GgezInteractionsEngine::with_capacity(32, 1024);
        Self {
            ui,
            ui_interactions,
            ui_resources,
        }
    }
}

impl EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.ui_interactions.update(ctx);
        self.ui.process();
        self.ui
            .interact(&mut self.ui_interactions)
            .expect("Could not interact with UI");
        Ok(())
    }

    fn text_input_event(&mut self, _: &mut Context, character: char) {
        self.ui_interactions.text_input_event(character);
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, _: KeyMods, _: bool) {
        if keycode == KeyCode::Escape {
            ggez::event::quit(ctx);
        }
        self.ui_interactions.key_down_event(keycode);
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let (width, height) = graphics::drawable_size(ctx);
        drop(graphics::set_screen_coordinates(
            ctx,
            graphics::Rect::new(0.0, 0.0, width, height),
        ));
        graphics::clear(ctx, graphics::WHITE);
        let ui_space = Rect {
            left: 0.0,
            right: width,
            top: 0.0,
            bottom: height,
        };
        self.ui
            .layout(ui_space, &mut DefaultLayoutEngine)
            .expect("UI could not layout widgets");
        self.ui
            .render(&mut GgezRenderer::new(ctx, &mut self.ui_resources))
            .expect("GGEZ renderer could not render UI");
        graphics::present(ctx)
    }
}
