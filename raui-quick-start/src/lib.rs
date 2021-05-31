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

use derive_builder::Builder;
use raui::prelude::*;
use raui_tetra_renderer::prelude::*;
use std::any::Any;
use tetra::{input::Key, Event};

pub use tetra;

pub type RauiQuickStartOnUpdate = Option<Box<dyn FnMut(&mut tetra::Context, &mut dyn Any) -> bool>>;
pub type RauiQuickStartOnEvent =
    Option<Box<dyn FnMut(&mut tetra::Context, tetra::Event, &mut dyn Any) -> bool>>;
pub type RauiQuickStartOnDraw = Option<Box<dyn FnMut(&mut tetra::Context, &mut dyn Any) -> bool>>;

/// The quick-start builder
#[derive(Builder)]
#[builder(default)]
pub struct RauiQuickStart {
    /// The title of the window
    window_title: String,
    /// The initial width and height of the window
    window_size: (i32, i32),
    /// The RAUI widget tree
    widget_tree: WidgetNode,
    /// Color of the window background.
    clear_color: Color,
    /// Tetra on update callback.
    #[builder(setter(skip))]
    on_update: RauiQuickStartOnUpdate,
    /// Tetra on event callback.
    #[builder(setter(skip))]
    on_event: RauiQuickStartOnEvent,
    /// Tetra on draw callback.
    #[builder(setter(skip))]
    on_draw: RauiQuickStartOnDraw,
}

impl Default for RauiQuickStart {
    fn default() -> Self {
        Self {
            window_title: "RAUI Quick Start".into(),
            window_size: (800, 600),
            widget_tree: widget!(()),
            clear_color: Color::default(),
            on_update: None,
            on_event: None,
            on_draw: None,
        }
    }
}

impl RauiQuickStart {
    pub fn on_update<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut tetra::Context, &mut dyn Any) -> bool + 'static,
    {
        self.on_update = Some(Box::new(f));
        self
    }

    pub fn on_event<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut tetra::Context, tetra::Event, &mut dyn Any) -> bool + 'static,
    {
        self.on_event = Some(Box::new(f));
        self
    }

    pub fn on_draw<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut tetra::Context, &mut dyn Any) -> bool + 'static,
    {
        self.on_draw = Some(Box::new(f));
        self
    }

    /// Run the app!
    ///
    /// Displays the window and runs the RAUI UI.
    pub fn run(self) -> Result<(), tetra::TetraError> {
        self.run_with_app_data(())
    }

    /// Run the app with additional application data!
    ///
    /// Displays the window and runs the RAUI UI. App data is used by RAUI as process context.
    pub fn run_with_app_data<T>(self, app_data: T) -> Result<(), tetra::TetraError>
    where
        T: 'static,
    {
        // Create a new tetra context, setting the wind,ow title and size
        tetra::ContextBuilder::new(&self.window_title, self.window_size.0, self.window_size.1)
            // Configure the tetra window options
            .resizable(true)
            .key_repeat(true)
            .show_mouse(true)
            .build()?
            // And run the Tetra game. We pass it a closure that returns our App
            .run(|context| {
                App::<T>::new(
                    context,
                    self.widget_tree,
                    tetra::graphics::Color::rgba(
                        self.clear_color.r,
                        self.clear_color.g,
                        self.clear_color.b,
                        self.clear_color.a,
                    ),
                    app_data,
                    self.on_update,
                    self.on_event,
                    self.on_draw,
                )
            })?;

        Ok(())
    }
}

/// Our App struct is responsible for handling the Tetra events and rendering
/// the GUI when necessary
struct App<T> {
    /// The UI field is a `TetraSimpleHost` which comes from the
    /// `raui_tetra_renderer` crate and helps wrap some the of the setup
    /// necessary to use RAUI with Tetra.
    ui: TetraSimpleHost,
    /// Color of the window background.
    clear_color: tetra::graphics::Color,
    /// App data used as process.
    app_data: T,
    /// Tetra on update callback.
    on_update: RauiQuickStartOnUpdate,
    /// Tetra on event callback.
    on_event: RauiQuickStartOnEvent,
    /// Tetra on draw callback.
    on_draw: RauiQuickStartOnDraw,
}

impl<T> App<T> {
    fn new(
        context: &mut tetra::Context,
        tree: WidgetNode,
        clear_color: tetra::graphics::Color,
        app_data: T,
        on_update: RauiQuickStartOnUpdate,
        on_event: RauiQuickStartOnEvent,
        on_draw: RauiQuickStartOnDraw,
    ) -> Result<Self, tetra::TetraError> {
        // Finally we need to provide a setup function that will initialize the
        // RAUI application created by the TetraSimpleHost. We will use the
        // default setup function provided by RAUI.
        let setup = raui::core::widget::setup;

        Ok(Self {
            // Create a TetraSimpleHost out of all the pieces we assigned above
            ui: TetraSimpleHost::new(context, tree, &[], &[], setup)?,
            clear_color,
            app_data,
            on_update,
            on_event,
            on_draw,
        })
    }
}

/// Finally, we need to hook into the Tetra events and drive the RAUI UI. For
/// each of these functions we just forward the arguments to the our
/// `TetraSimpleHost` and let it paint the UI on the screen.
impl<T> tetra::State for App<T>
where
    T: 'static,
{
    fn update(&mut self, ctx: &mut tetra::Context) -> Result<(), tetra::TetraError> {
        if let Some(callback) = &mut self.on_update {
            if callback(ctx, &mut self.app_data) {
                self.ui.application.mark_dirty();
            }
        }
        self.ui.update_with_context(ctx, &mut self.app_data);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut tetra::Context) -> Result<(), tetra::TetraError> {
        // Clear the screen white first
        tetra::graphics::clear(ctx, self.clear_color);
        if let Some(callback) = &mut self.on_draw {
            if callback(ctx, &mut self.app_data) {
                self.ui.application.mark_dirty();
            }
        }
        // Then draw the UI
        self.ui.draw(ctx, PrintLogger)?;
        Ok(())
    }

    fn event(
        &mut self,
        ctx: &mut tetra::Context,
        event: tetra::Event,
    ) -> Result<(), tetra::TetraError> {
        if let Some(callback) = &mut self.on_event {
            if callback(ctx, event.clone(), &mut self.app_data) {
                self.ui.application.mark_dirty();
            }
        }

        self.ui.event(ctx, &event);
        if let Event::KeyPressed { key: Key::F2 } = event {
            println!("LAYOUT: {:#?}", self.ui.application.layout_data());
        }
        if let Event::KeyPressed { key: Key::F3 } = event {
            println!("INTERACTIONS: {:#?}", self.ui.interactions);
        }
        if let Event::KeyPressed { key: Key::F4 } = event {
            println!(
                "INSPECT TREE: {:#?}",
                self.ui.application.rendered_tree().inspect()
            );
        }
        Ok(())
    }
}
