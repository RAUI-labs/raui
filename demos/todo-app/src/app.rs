use crate::ui::components::app::{app, AppMessage, AppState};
use ggez::{
    event::EventHandler,
    graphics,
    input::keyboard::{KeyCode, KeyMods},
    Context, GameResult,
};
use raui_core::{application::Application as UI, prelude::*};
use raui_ggez_renderer::prelude::*;
use raui_material::setup as setup_material;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{read_to_string, write},
};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct AssetsManifest {
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub fonts: HashMap<String, String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub images: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum AppSignal {
    Ready(WidgetId),
    Save(AppState),
}

pub struct App {
    ui: UI,
    ui_interactions: GgezInteractionsEngine,
    ui_resources: GgezResources,
}

impl App {
    pub fn new(ctx: &mut Context) -> Self {
        let ui_resources = Self::initialize_resources(ctx);
        let mut ui = UI::new();
        ui.setup(setup);
        ui.setup(setup_material);
        ui.apply(widget! { (#{"app"} app) });
        let ui_interactions = GgezInteractionsEngine::with_capacity(32, 1024);
        Self {
            ui,
            ui_interactions,
            ui_resources,
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
            ui_resources.images.insert(
                key.to_owned(),
                graphics::Image::new(ctx, path)
                    .unwrap_or_else(|_| panic!("GGEZ could not load `{}`!", path)),
            );
        }
        ui_resources
    }

    fn load(&mut self, id: &WidgetId) {
        if let Ok(content) = read_to_string("./state.json") {
            if let Ok(state) = serde_json::from_str(&content) {
                self.ui.send_message(id, AppMessage::Load(state));
            }
        }
    }

    fn save(state: &AppState) {
        if let Ok(content) = serde_json::to_string_pretty(state) {
            drop(write("./state.json", content));
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
        self.ui.process();
        self.ui
            .interact(&mut self.ui_interactions)
            .expect("Could not interact with UI");
        for (_, data) in self.ui.consume_signals() {
            if let Some(signal) = data.downcast_ref::<AppSignal>() {
                match signal {
                    AppSignal::Ready(id) => self.load(id),
                    AppSignal::Save(state) => Self::save(state),
                }
            }
        }
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
        graphics::clear(ctx, graphics::BLACK);
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
