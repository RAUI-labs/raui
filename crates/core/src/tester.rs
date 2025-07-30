use crate::{
    application::Application,
    interactive::default_interactions_engine::DefaultInteractionsEngine,
    layout::{CoordsMapping, default_layout_engine::DefaultLayoutEngine},
};

pub trait AppCycleFrameRunner<T> {
    fn run_frame(self, tester: &mut AppCycleTester<T>);
}

impl<T> AppCycleFrameRunner<T> for () {
    fn run_frame(self, _: &mut AppCycleTester<T>) {}
}

impl<T, F> AppCycleFrameRunner<T> for F
where
    F: FnMut(&mut AppCycleTester<T>),
{
    fn run_frame(mut self, tester: &mut AppCycleTester<T>) {
        (self)(tester);
    }
}

pub struct AppCycleTester<T> {
    pub coords_mapping: CoordsMapping,
    pub application: Application,
    pub layout_engine: DefaultLayoutEngine,
    pub interactions_engine: DefaultInteractionsEngine,
    pub user_data: T,
}

impl<T> AppCycleTester<T> {
    pub fn new(coords_mapping: CoordsMapping, user_data: T) -> Self {
        Self {
            coords_mapping,
            application: Default::default(),
            layout_engine: Default::default(),
            interactions_engine: Default::default(),
            user_data,
        }
    }

    pub fn run_frame(&mut self, frame_runner: impl AppCycleFrameRunner<T>) {
        frame_runner.run_frame(self);
        if self.application.process() {
            self.application
                .layout(&self.coords_mapping, &mut self.layout_engine)
                .unwrap();
        }
        self.application
            .interact(&mut self.interactions_engine)
            .unwrap();
    }
}
