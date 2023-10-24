use crate::{app::SharedApp, Vertex};
use glutin::event::Event;
use raui_core::{
    application::Application,
    make_widget,
    tester::{AppCycleFrameRunner, AppCycleTester},
    widget::{component::containers::content_box::content_box, utils::Color},
};
use raui_immediate::{make_widgets, ImmediateContext};
use spitfire_fontdue::TextRenderer;
use spitfire_glow::{
    app::{App, AppConfig, AppState},
    graphics::Graphics,
};

#[derive(Default)]
pub struct ImmediateApp {
    shared: SharedApp,
}

impl ImmediateApp {
    pub fn simple(title: impl ToString, callback: impl FnMut() + 'static) {
        App::<Vertex>::new(AppConfig::default().title(title)).run(Self::default().update(callback));
    }

    pub fn simple_fullscreen(title: impl ToString, callback: impl FnMut() + 'static) {
        App::<Vertex>::new(AppConfig::default().title(title).fullscreen(true))
            .run(Self::default().update(callback));
    }

    pub fn test_frame<F: FnMut()>(f: F) -> ImmediateAppCycleFrameRunner<F> {
        ImmediateAppCycleFrameRunner(f)
    }

    pub fn update(mut self, callback: impl FnMut() + 'static) -> Self {
        let mut callback = Box::new(callback);
        let context = ImmediateContext::default();
        self.shared.on_update = Some(Box::new(move |application| {
            raui_immediate::reset();
            let widgets = make_widgets(&context, || {
                callback();
            });
            application.apply(make_widget!(content_box).listed_slots(widgets.into_iter()));
        }));
        self
    }

    pub fn redraw(
        mut self,
        f: impl FnMut(f32, &mut Graphics<Vertex>, &mut TextRenderer<Color>) + 'static,
    ) -> Self {
        self.shared.on_redraw = Some(Box::new(f));
        self
    }

    pub fn event(mut self, f: impl FnMut(&mut Application, Event<()>) -> bool + 'static) -> Self {
        self.shared.on_event = Some(Box::new(f));
        self
    }

    pub fn setup(mut self, mut f: impl FnMut(&mut Application)) -> Self {
        f(&mut self.shared.application);
        self
    }
}

impl AppState<Vertex> for ImmediateApp {
    fn on_init(&mut self, graphics: &mut Graphics<Vertex>) {
        self.shared.init(graphics);
    }

    fn on_redraw(&mut self, graphics: &mut Graphics<Vertex>) {
        self.shared.redraw(graphics);
    }

    fn on_event(&mut self, event: Event<()>) -> bool {
        self.shared.event(event)
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
            .apply(make_widget!(content_box).listed_slots(widgets.into_iter()));
    }
}
