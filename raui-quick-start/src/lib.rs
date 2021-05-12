//! Tiny library to get you started learning RAUI as quick as possible 🚀
//!
//! # Example
//!
//! ```no_run
//! # use raui::prelude::*;
//! # fn my_widget(_: WidgetContext) -> WidgetNode { widget!(()) }
//! // Import the builder
//! use raui_quick_start::RauiQuickStartBuilder;
//!
//! // Create the builder
//! RauiQuickStartBuilder::default()
//!     // Set our window title
//!     .window_title("My RAUI App".into())
//!     // Set the RAUI widget tree for our app
//!     .widget_tree(widget! {
//!         (my_widget)
//!     })
//!     // Build the app
//!     .build()
//!     .expect("Error building quick start")
//!     // And run it! 🚀
//!     .run()
//!     .expect("Error running RAUI app");
//! ```

use raui::prelude::*;
use raui_tetra_renderer::prelude::*;

use derive_builder::Builder;

/// The quick-start builder
#[derive(Builder, Clone)]
#[builder(default)]
pub struct RauiQuickStart {
    /// The title of the window
    window_title: String,
    /// The initial width and height of the window
    window_size: (i32, i32),
    /// The RAUI widget tree
    widget_tree: WidgetNode,
}

impl Default for RauiQuickStart {
    fn default() -> Self {
        Self {
            window_title: "RAUI Quick Start".into(),
            window_size: (800, 600),
            widget_tree: widget!(()),
        }
    }
}

impl RauiQuickStart {
    /// Run the app!
    ///
    /// Displays the window and runs the RAUI UI.
    pub fn run(self) -> Result<(), tetra::TetraError> {
        // Create a new tetra context, setting the wind,ow title and size
        tetra::ContextBuilder::new(&self.window_title, self.window_size.0, self.window_size.1)
            // Configure the tetra window options
            .resizable(true)
            .key_repeat(true)
            .show_mouse(true)
            .build()?
            // And run the Tetra game. We pass it a closure that returns our App
            .run(|context| App::new(context, self.widget_tree))?;

        Ok(())
    }
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
    fn new(context: &mut tetra::Context, tree: WidgetNode) -> Result<Self, tetra::TetraError> {
        // Finally we need to provide a setup function that will initialize the
        // RAUI application created by the TetraSimpleHost. We will use the
        // default setup function provided by RAUI.
        let setup = raui::core::widget::setup;

        Ok(Self {
            // Create a TetraSimpleHost out of all the pieces we assigned above
            ui: TetraSimpleHost::new(context, tree, &[], &[], setup)?,
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
        // Clear the screen white first
        tetra::graphics::clear(ctx, tetra::graphics::Color::WHITE);
        // Then draw the UI
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
