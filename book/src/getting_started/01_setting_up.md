# Setting Up

We're going to get setup with a blank Tetra game project with RAUI hooked up to render to it.

## Creating the Project

First we need to create a new Rust project and get our dependencies setup.

Create a new cargo project:

```bash
cargo new --bin guide
```

Then add the following dependencies to the `Cargo.toml`:

```toml
[dependencies]
# The sdl2_bundled feature tells the crate to download and install SDL2 so you
# don't have to have it installed
tetra = { version = "0.6", features = ["sdl2_bundled"] }
raui = "0.29"
raui-tetra-renderer = "0.29"
```

## Initializing Tetra

Next we need to setup our Tetra game window with the RAUI renderer hooked up to it.

```rust
use raui::prelude::*;
use raui_tetra_renderer::simple_host::TetraSimpleHost;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new tetra context, setting the window title and size
    tetra::ContextBuilder::new("Hello RAUI!", 800, 600)
        // Configure the tetra window options
        .resizable(true)
        .key_repeat(true)
        .show_mouse(true)
        .build()?
        // And run the Tetra game. We pass it a closure that returns our App
        .run(|context| App::new(context))?;

    Ok(())
}

/// Our App struct is responsible for handling the Tetra events and rendering
/// the GUI when necessary
struct App {
    /// The UI field is a `TetraSimpleHost` which comes from the
    /// `raui_tetra_renderer` crate and helps wrap some the of the setup
    /// necessary to use RAUI with Tetra.
    ui: TetraSimpleHost,
}

impl App {
    fn new(context: &mut tetra::Context) -> Result<Self, tetra::TetraError> {
        // This is where we pass in our actual RAUI widget tree, specifying the
        // whole RAUI UI. The widget! macro is used to create widget trees. For
        // now we just provide an empty tree
        let tree = widget! {
            ()
        };

        // We need to provide the list of font resources to load
        let fonts = &[
            // For every font we provide 4 fields
            (
                // The font ID, used to reference the font in the widget tree
                "verdana",
                // The size of the font
                32,
                // The scale of the font
                1.0,
                // The path to the font
                "./resources/verdana.ttf",
            ),
        ];

        // And the list of image resources to load
        let images = &[
            // For every image we need we provide the ID used to reference it
            // and the path to the image file
            ("cat", "./resources/cat.jpg"),
            ("cats", "./resources/cats.jpg"),
        ];

        // Finally we need to provide a setup function that will initialize the
        // RAUI application created by the TetraSimpleHost. We will use the
        // default setup function provided by RAUI.
        let setup = raui::core::widget::setup;

        Ok(Self {
            // Create a TetraSimpleHost out of all the pieces we assigned above
            ui: TetraSimpleHost::new(context, tree, fonts, images, setup)?,
        })
    }
}

/// Finally, we need to hook into the Tetra events and drive the RAUI UI. For
/// each of these functions we just forward the arguments to the our
/// `TetraSimpleHost` and let it paint the UI on the screen.
impl tetra::State for App {
    fn update(&mut self, ctx: &mut tetra::Context) -> Result<(), tetra::TetraError> {
        self.ui.update(ctx);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut tetra::Context) -> Result<(), tetra::TetraError> {
        self.ui.draw(ctx)?;
        Ok(())
    }

    fn event(
        &mut self,
        ctx: &mut tetra::Context,
        event: tetra::Event,
    ) -> Result<(), tetra::TetraError> {
        self.ui.event(ctx, &event);
        Ok(())
    }
}
```

## Summary

At this point you should be able to `cargo run` and have a blank window pop up, but we're not here for a blank window, so let's go put some GUI on the screen!
