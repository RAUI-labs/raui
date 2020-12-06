#[macro_use]
extern crate raui_core;

mod app;
mod components;

use crate::app::App;
use ggez::{event, ContextBuilder};

fn main() {
    let (mut ctx, mut event_loop) = ContextBuilder::new("TODO App", "Cool Game Author")
        .build()
        .expect("Could not create GGEZ context");
    let mut app = App::new();
    if let Err(error) = event::run(&mut ctx, &mut event_loop, &mut app) {
        println!("Error: {}", error)
    }
}
