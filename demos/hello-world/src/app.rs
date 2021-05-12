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
                // We can preload certain resources if desired. In this case we preload a font. You
                // can mix preloaded and on-demand loaded fonts and textures without any problems.
                &[PreloadedFont {
                    // In the UI code will use this name to refer to the preloaded font, instead of
                    // the full file path
                    id: "verdana",
                    size: 32,
                    scale: 1.0,
                    path: "./resources/verdana.ttf",
                }],
                // Let's load our textures on demand. To do that we leave the preloaded textures
                // empty. In the UI code we will refer to texture images using their full file path.
                &[],
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
