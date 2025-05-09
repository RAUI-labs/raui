use crate::{Vertex, app::SharedApp, interactions::AppInteractionsEngine};
use glutin::{event::Event, window::Window};
use raui_core::{
    application::Application,
    interactive::default_interactions_engine::DefaultInteractionsEngine,
    layout::CoordsMappingScaling,
    make_widget,
    tester::{AppCycleFrameRunner, AppCycleTester},
    widget::{component::containers::content_box::content_box, utils::Color},
};
use raui_immediate::{ImmediateContext, make_widgets};
use spitfire_fontdue::TextRenderer;
use spitfire_glow::{
    app::{App, AppConfig, AppControl, AppState},
    graphics::Graphics,
};

#[derive(Default)]
pub struct ImmediateApp {
    shared: SharedApp,
}

impl ImmediateApp {
    pub fn simple(title: impl ToString, callback: impl FnMut(&mut AppControl) + 'static) {
        App::<Vertex>::new(AppConfig::default().title(title)).run(Self::default().update(callback));
    }

    pub fn simple_scaled(
        title: impl ToString,
        scaling: CoordsMappingScaling,
        callback: impl FnMut(&mut AppControl) + 'static,
    ) {
        App::<Vertex>::new(AppConfig::default().title(title)).run(
            Self::default()
                .coords_mapping_scaling(scaling)
                .update(callback),
        );
    }

    pub fn simple_fullscreen(
        title: impl ToString,
        callback: impl FnMut(&mut AppControl) + 'static,
    ) {
        App::<Vertex>::new(AppConfig::default().title(title).fullscreen(true))
            .run(Self::default().update(callback));
    }

    pub fn simple_fullscreen_scaled(
        title: impl ToString,
        scaling: CoordsMappingScaling,
        callback: impl FnMut(&mut AppControl) + 'static,
    ) {
        App::<Vertex>::new(AppConfig::default().title(title).fullscreen(true)).run(
            Self::default()
                .coords_mapping_scaling(scaling)
                .update(callback),
        );
    }

    pub fn test_frame<F: FnMut()>(f: F) -> ImmediateAppCycleFrameRunner<F> {
        ImmediateAppCycleFrameRunner(f)
    }

    pub fn update(mut self, callback: impl FnMut(&mut AppControl) + 'static) -> Self {
        let mut callback = Box::new(callback);
        let context = ImmediateContext::default();
        self.shared.on_update = Some(Box::new(move |application, control| {
            raui_immediate::reset();
            let widgets = make_widgets(&context, || {
                callback(control);
            });
            application.apply(make_widget!(content_box).listed_slots(widgets));
        }));
        self
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

    pub fn coords_mapping_scaling(mut self, value: CoordsMappingScaling) -> Self {
        self.shared.coords_mapping_scaling = value;
        self
    }
}

impl AppState<Vertex> for ImmediateApp {
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

pub struct ImmediateAppCycleFrameRunner<F: FnMut()>(F);

impl<F: FnMut()> AppCycleFrameRunner<ImmediateContext> for ImmediateAppCycleFrameRunner<F> {
    fn run_frame(mut self, tester: &mut AppCycleTester<ImmediateContext>) {
        raui_immediate::reset();
        let widgets = make_widgets(&tester.user_data, || {
            (self.0)();
        });
        tester
            .application
            .apply(make_widget!(content_box).listed_slots(widgets));
    }
}
