use crate::{app::SharedApp, Vertex};
use glutin::{event::Event, window::Window};
use raui_core::{
    application::Application,
    view_model::ViewModel,
    widget::{node::WidgetNode, utils::Color},
};
use spitfire_fontdue::TextRenderer;
use spitfire_glow::{
    app::{App, AppConfig, AppState},
    graphics::Graphics,
};

#[derive(Default)]
pub struct DeclarativeApp {
    shared: SharedApp,
}

impl DeclarativeApp {
    pub fn simple(title: impl ToString, root: impl Into<WidgetNode>) {
        App::<Vertex>::new(AppConfig::default().title(title)).run(Self::default().tree(root));
    }

    pub fn simple_fullscreen(title: impl ToString, root: impl Into<WidgetNode>) {
        App::<Vertex>::new(AppConfig::default().title(title).fullscreen(true))
            .run(Self::default().tree(root));
    }

    pub fn update(mut self, f: impl FnMut(&mut Application) + 'static) -> Self {
        self.shared.on_update = Some(Box::new(f));
        self
    }

    pub fn redraw(
        mut self,
        f: impl FnMut(f32, &mut Graphics<Vertex>, &mut TextRenderer<Color>) + 'static,
    ) -> Self {
        self.shared.on_redraw = Some(Box::new(f));
        self
    }

    pub fn event(
        mut self,
        f: impl FnMut(&mut Application, Event<()>, &mut Window) -> bool + 'static,
    ) -> Self {
        self.shared.on_event = Some(Box::new(f));
        self
    }

    pub fn setup(mut self, mut f: impl FnMut(&mut Application)) -> Self {
        f(&mut self.shared.application);
        self
    }

    pub fn view_model(mut self, name: impl ToString, view_model: ViewModel) -> Self {
        self.shared
            .application
            .view_models
            .insert(name.to_string(), view_model);
        self
    }

    pub fn tree(mut self, root: impl Into<WidgetNode>) -> Self {
        self.shared.application.apply(root);
        self
    }
}

impl AppState<Vertex> for DeclarativeApp {
    fn on_init(&mut self, graphics: &mut Graphics<Vertex>) {
        self.shared.init(graphics);
    }

    fn on_redraw(&mut self, graphics: &mut Graphics<Vertex>) {
        self.shared.redraw(graphics);
    }

    fn on_event(&mut self, event: Event<()>, window: &mut Window) -> bool {
        self.shared.event(event, window)
    }
}
