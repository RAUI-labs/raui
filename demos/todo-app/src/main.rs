mod app;
mod ui;

use crate::app::TodoState;
use tetra::ContextBuilder;

fn main() -> tetra::Result {
    ContextBuilder::new("TODO App", 800, 600)
        .resizable(true)
        .key_repeat(true)
        .show_mouse(true)
        .build()?
        .run(TodoState::new)
}
