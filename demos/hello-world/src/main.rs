mod ui;

use crate::ui::components::{app::app, content::content, title_bar::title_bar};
use raui::prelude::*;

fn main() {
    DeclarativeApp::simple(
        "Hello World!",
        make_widget!(app)
            .named_slot("title", make_widget!(title_bar))
            .named_slot("content", make_widget!(content)),
    );
}
