use crate::ui::components::{app::app, content::content, title_bar::title_bar};
use raui_core::prelude::*;
use raui_tetra_renderer::prelude::*;
use tetra::{
    graphics::{self, Color},
    input::{is_key_modifier_down, Key, KeyModifier},
    Context, Event, State,
};

pub struct AppState {
    ui: TetraSimpleHost,
}

impl AppState {
    pub fn new(context: &mut Context) -> tetra::Result<Self> {
        let tree = widget! {
            (app {
                title = (title_bar)
                content = (content)
            })
        };
        Ok(Self {
            ui: TetraSimpleHost::new(
                context,
                tree,
                &[("verdana", 32, 1.0, "./resources/verdana.ttf")],
                &[
                    ("cat", "./resources/cat.jpg"),
                    ("cats", "./resources/cats.jpg"),
                ],
                setup,
            )?,
        })
    }
}

impl State for AppState {
    fn update(&mut self, context: &mut Context) -> tetra::Result {
        self.ui.update(context);
        Ok(())
    }

    fn draw(&mut self, context: &mut Context) -> tetra::Result {
        graphics::clear(context, Color::WHITE);
        self.ui.draw(context)?;
        Ok(())
    }

    fn event(&mut self, context: &mut Context, event: Event) -> tetra::Result {
        self.ui.event(context, &event);
        if let Event::KeyPressed { key: Key::P } = event {
            if is_key_modifier_down(context, KeyModifier::Ctrl) {
                println!("LAYOUT: {:#?}", self.ui.application.layout_data());
                if is_key_modifier_down(context, KeyModifier::Shift) {
                    println!("INTERACTIONS: {:#?}", self.ui.interactions);
                }
            }
        }
        Ok(())
    }
}
