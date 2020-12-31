#[macro_use]
extern crate raui_core;

mod app;
mod ui;

use crate::app::App;
use ggez::{event, ContextBuilder};

fn main() {
    let resource_dir = if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = std::path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        std::path::PathBuf::from("./resources")
    };
    let (mut ctx, mut event_loop) = ContextBuilder::new("In-Game Demo", "Cool Game Author")
        .add_resource_path(resource_dir)
        .window_mode(
            ggez::conf::WindowMode::default()
                .resizable(true)
                .maximized(false),
        )
        .build()
        .expect("Could not create GGEZ context");
    let mut app = App::new(&mut ctx);
    if let Err(error) = event::run(&mut ctx, &mut event_loop, &mut app) {
        println!("Error: {}", error);
    }
}
