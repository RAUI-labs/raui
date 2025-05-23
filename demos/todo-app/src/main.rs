mod model;
mod ui;

use crate::{model::AppState, ui::components::app::app};
use raui::{
    app::app::{App, AppConfig, declarative::DeclarativeApp},
    core::{make_widget, view_model::ViewModel},
};

fn main() {
    let app = DeclarativeApp::default()
        .tree(make_widget!(app))
        .view_model(
            AppState::VIEW_MODEL,
            ViewModel::produce(|properties| {
                let mut result = AppState::new(properties);
                result.load();
                result
            }),
        );

    App::new(AppConfig::default().title("TODO App")).run(app);
}
