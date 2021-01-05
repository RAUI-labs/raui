use crate::ui::components::{app::app, content::content, title_bar::title_bar};
use ggez::{
    event::EventHandler,
    graphics,
    input::keyboard::{KeyCode, KeyMods},
    timer, Context, GameResult,
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
        ui.setup(setup);
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

    fn make_coords_mapping(ctx: &Context) -> CoordsMapping {
        let (width, height) = graphics::drawable_size(ctx);
        let area = Rect {
            left: 0.0,
            right: width,
            top: 0.0,
            bottom: height,
        };
        CoordsMapping::new(area)
    }
}

impl EventHandler for App {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mapping = Self::make_coords_mapping(ctx);
        self.ui_interactions.update(ctx, &mapping);
        self.ui.animations_delta_time = timer::delta(ctx).as_secs_f32();
        self.ui.process();
        self.ui
            .interact(&mut self.ui_interactions)
            .expect("Could not interact with UI");
        Ok(())
    }

    fn text_input_event(&mut self, _: &mut Context, character: char) {
        self.ui_interactions.text_input_event(character);
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, keymods: KeyMods, _: bool) {
        if keycode == KeyCode::Escape {
            ggez::event::quit(ctx);
        }
        self.ui_interactions.key_down_event(keycode);
        if keycode == KeyCode::P && keymods.contains(KeyMods::CTRL) {
            println!("LAYOUT: {:#?}", self.ui.layout_data());
        }
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let (width, height) = graphics::drawable_size(ctx);
        drop(graphics::set_screen_coordinates(
            ctx,
            graphics::Rect::new(0.0, 0.0, width, height),
        ));
        graphics::clear(ctx, graphics::WHITE);
        let mapping = Self::make_coords_mapping(ctx);
        self.ui
            .layout(&mapping, &mut DefaultLayoutEngine)
            .expect("UI could not layout widgets");
        self.ui
            .render(
                &mapping,
                &mut GgezRenderer::new(ctx, &mut self.ui_resources),
            )
            .expect("GGEZ renderer could not render UI");
        graphics::present(ctx)
    }
}
