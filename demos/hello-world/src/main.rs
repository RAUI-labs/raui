mod app;
mod ui;

use crate::app::AppState;
use tetra::ContextBuilder;

fn main() -> tetra::Result {
    ContextBuilder::new("Hello, Tetra!", 800, 600)
        .resizable(true)
        .key_repeat(true)
        .show_mouse(true)
        .build()?
        .run(|context| AppState::new(context))
}
