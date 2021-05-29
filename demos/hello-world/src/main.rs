use raui::prelude::*;
use raui_quick_start::RauiQuickStartBuilder;

mod ui;

use ui::components::{app::app, content::content, title_bar::title_bar};

fn main() {
    RauiQuickStartBuilder::default()
        .window_title("Hello World!".to_owned())
        .widget_tree(widget! {
            (app {
                title = (title_bar)
                content = (content)
            })
        })
        .build()
        .unwrap()
        .run()
        .unwrap();
}
