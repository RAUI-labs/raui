use crate::{Vertex, app::SharedApp, interactions::AppInteractionsEngine};
use glutin::{event::Event, window::Window};
use raui_core::{
    application::{Application, ChangeNotifier},
    interactive::default_interactions_engine::DefaultInteractionsEngine,
    layout::CoordsMappingScaling,
    widget::utils::Color,
};
use raui_retained::{View, ViewState};
use spitfire_fontdue::TextRenderer;
use spitfire_glow::{
    app::{App, AppConfig, AppControl, AppState},
    graphics::Graphics,
};

pub struct RetainedApp<T: ViewState> {
    shared: SharedApp,
    root: Option<View<T>>,
}

impl<T: ViewState> Default for RetainedApp<T> {
    fn default() -> Self {
        Self {
            shared: Default::default(),
            root: None,
        }
    }
}

impl<T: ViewState> RetainedApp<T> {
    pub fn simple(title: impl ToString, producer: impl FnMut(ChangeNotifier) -> View<T>) {
        App::<Vertex>::new(AppConfig::default().title(title)).run(Self::default().tree(producer));
    }

    pub fn simple_scaled(
        title: impl ToString,
        scaling: CoordsMappingScaling,
        producer: impl FnMut(ChangeNotifier) -> View<T>,
    ) {
        App::<Vertex>::new(AppConfig::default().title(title)).run(
            Self::default()
                .coords_mapping_scaling(scaling)
                .tree(producer),
        );
    }

    pub fn simple_fullscreen(
        title: impl ToString,
        producer: impl FnMut(ChangeNotifier) -> View<T>,
    ) {
        App::<Vertex>::new(AppConfig::default().title(title).fullscreen(true))
            .run(Self::default().tree(producer));
    }

    pub fn simple_fullscreen_scaled(
        title: impl ToString,
        scaling: CoordsMappingScaling,
        producer: impl FnMut(ChangeNotifier) -> View<T>,
    ) {
        App::<Vertex>::new(AppConfig::default().title(title).fullscreen(true)).run(
            Self::default()
                .coords_mapping_scaling(scaling)
                .tree(producer),
        );
    }

    pub fn redraw(
        mut self,
        f: impl FnMut(f32, &mut Graphics<Vertex>, &mut TextRenderer<Color>, &mut AppControl) + 'static,
    ) -> Self {
        self.shared.on_redraw = Some(Box::new(f));
        self
    }

    pub fn event(
        mut self,
        f: impl FnMut(&mut Application, Event<()>, &mut Window, &mut DefaultInteractionsEngine) -> bool
        + 'static,
    ) -> Self {
        self.shared.on_event = Some(Box::new(f));
        self
    }

    pub fn setup(mut self, mut f: impl FnMut(&mut Application)) -> Self {
        f(&mut self.shared.application);
        self
    }

    pub fn setup_interactions(mut self, mut f: impl FnMut(&mut AppInteractionsEngine)) -> Self {
        f(&mut self.shared.interactions);
        self
    }

    pub fn tree(mut self, mut producer: impl FnMut(ChangeNotifier) -> View<T>) -> Self {
        let root = producer(self.shared.application.notifier());
        self.shared.application.apply(root.component().key("root"));
        self.root = Some(root);
        self
    }

    pub fn coords_mapping_scaling(mut self, value: CoordsMappingScaling) -> Self {
        self.shared.coords_mapping_scaling = value;
        self
    }
}

impl<T: ViewState> AppState<Vertex> for RetainedApp<T> {
    fn on_init(&mut self, graphics: &mut Graphics<Vertex>, _: &mut AppControl) {
        self.shared.init(graphics);
    }

    fn on_redraw(&mut self, graphics: &mut Graphics<Vertex>, control: &mut AppControl) {
        self.shared.redraw(graphics, control);
    }

    fn on_event(&mut self, event: Event<()>, window: &mut Window) -> bool {
        self.shared.event(event, window)
    }
}
