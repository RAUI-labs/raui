mod ui;

use crate::ui::{
    components::{app::app, content::content, title_bar::title_bar},
    view_models::AppData,
};
use raui::{
    app::app::{App, AppConfig, declarative::DeclarativeApp},
    core::{make_widget, view_model::ViewModel},
};

fn main() {
    let app = DeclarativeApp::default()
        .view_model(AppData::VIEW_MODEL, ViewModel::produce(AppData::new))
        .tree(
            make_widget!(app)
                .named_slot("title", make_widget!(title_bar))
                .named_slot("content", make_widget!(content)),
        );

    App::new(AppConfig::default().title("Hello World!")).run(app);
}
