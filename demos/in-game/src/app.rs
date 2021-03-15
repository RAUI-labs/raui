use crate::ui::components::app::{app, AppProps};
use ggez::{
    event::EventHandler,
    graphics::{self, DrawParam, FilterMode, Image},
    input::keyboard::{KeyCode, KeyMods},
    timer, Context, GameResult,
};
use lipsum::MarkovChain;
use raui_core::{application::Application as UI, prelude::*};
use raui_ggez_renderer::prelude::*;
use raui_material::setup as setup_material;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::read_to_string};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct AssetsManifest {
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub fonts: HashMap<String, String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub images: HashMap<String, String>,
}

pub struct App {
    ui: UI,
    ui_interactions: GgezInteractionsEngine,
    ui_resources: GgezResources,
    mockup_image: Image,
}

impl App {
    pub fn new(ctx: &mut Context) -> Self {
        let mut mockup_image = Image::new(ctx, "/images/in-game-mockup.png")
            .expect("Could not load `/images/in-game-mockup.png`!");
        mockup_image.set_filter(FilterMode::Nearest);
        let ui_resources = Self::initialize_resources(ctx);
        let mut ui = UI::new();
        ui.setup(setup);
        ui.setup(setup_material);
        let text = read_to_string("./resources/text.txt").expect("Could not load `/text.txt`!");
        let mut markov = MarkovChain::new();
        markov.learn(&text);
        let props = AppProps {
            texts: (0..=18).map(|_| markov.generate(20)).collect::<Vec<_>>(),
        };
        ui.apply(widget! { (#{"app"} app: {props}) });
        let mut ui_interactions =
            GgezInteractionsEngine::with_capacity(0, 1024, 32, 32, 32, 32, 16);
        ui_interactions.engine.deselect_when_no_button_found = true;
        Self {
            ui,
            ui_interactions,
            ui_resources,
            mockup_image,
        }
    }

    fn initialize_resources(ctx: &mut Context) -> GgezResources {
        let mut ui_resources = GgezResources::default();
        let assets = serde_json::from_str::<AssetsManifest>(
            &read_to_string("./resources/assets.json").expect("Could not load assets manifest"),
        )
        .expect("Could not parse assets manifest");
        for (key, path) in &assets.fonts {
            ui_resources.fonts.insert(
                key.to_owned(),
                graphics::Font::new(ctx, path)
                    .unwrap_or_else(|_| panic!("GGEZ could not load `{}`!", path)),
            );
        }
        for (key, path) in &assets.images {
            let mut image = graphics::Image::new(ctx, path)
                .unwrap_or_else(|_| panic!("GGEZ could not load `{}`!", path));
            image.set_filter(FilterMode::Nearest);
            ui_resources.images.insert(key.to_owned(), image);
        }
        ui_resources
    }

    fn make_coords_mapping(ctx: &Context) -> CoordsMapping {
        let (width, height) = graphics::drawable_size(ctx);
        let area = Rect {
            left: 0.0,
            right: width,
            top: 0.0,
            bottom: height,
        };
        const SCALE: Scalar = 256.0;
        let scaling = CoordsMappingScaling::FitMinimum(Vec2 { x: SCALE, y: SCALE });
        CoordsMapping::new_scaling(area, scaling)
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
        self.ui.consume_signals();
        Ok(())
    }

    fn text_input_event(&mut self, _: &mut Context, character: char) {
        self.ui_interactions.text_input_event(character);
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, keymods: KeyMods, _: bool) {
        if keycode == KeyCode::Q && self.ui_interactions.engine.focused_text_input().is_none() {
            ggez::event::quit(ctx);
        }
        self.ui_interactions.key_down_event(keycode, keymods);
        if keycode == KeyCode::P && keymods.contains(KeyMods::CTRL) {
            println!("LAYOUT: {:#?}", self.ui.layout_data());
            if keymods.contains(KeyMods::SHIFT) {
                println!("INTERACTIONS: {:#?}", self.ui_interactions);
            }
        }
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let (width, height) = graphics::drawable_size(ctx);
        drop(graphics::set_screen_coordinates(
            ctx,
            graphics::Rect::new(0.0, 0.0, width, height),
        ));
        graphics::clear(ctx, graphics::BLACK);
        {
            let w = self.mockup_image.width() as Scalar;
            let h = self.mockup_image.height() as Scalar;
            let ra = width / height;
            let va = w / h;
            let scale = if ra >= va { width / w } else { height / h };
            let ox = (width - w * scale) * 0.5;
            let oy = (height - h * scale) * 0.5;
            graphics::draw(
                ctx,
                &self.mockup_image,
                DrawParam::default().dest([ox, oy]).scale([scale, scale]),
            )?;
        }
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
