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
use tetra::{input::Key, Event};

pub use tetra;

pub type RauiQuickStartOnUpdate =
    Option<Box<dyn FnMut(&mut tetra::Context, &mut TetraSimpleHost) -> bool>>;
pub type RauiQuickStartOnEvent =
    Option<Box<dyn FnMut(&mut tetra::Context, &mut TetraSimpleHost, tetra::Event) -> bool>>;
pub type RauiQuickStartOnDraw =
    Option<Box<dyn FnMut(&mut tetra::Context, &mut TetraSimpleHost) -> bool>>;

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
    /// View-model collection.
    #[builder(setter(skip))]
    view_models: ViewModelCollection,
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
            view_models: Default::default(),
        }
    }
}

impl RauiQuickStart {
    pub fn on_update<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut tetra::Context, &mut TetraSimpleHost) -> bool + 'static,
    {
        self.on_update = Some(Box::new(f));
        self
    }

    pub fn on_event<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut tetra::Context, &mut TetraSimpleHost, tetra::Event) -> bool + 'static,
    {
        self.on_event = Some(Box::new(f));
        self
    }

    pub fn on_draw<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut tetra::Context, &mut TetraSimpleHost) -> bool + 'static,
    {
        self.on_draw = Some(Box::new(f));
        self
    }

    pub fn view_model_raw(mut self, name: impl ToString, view_model: ViewModel) -> Self {
        self.view_models.insert(name.to_string(), view_model);
        self
    }

    pub fn view_model<T: 'static>(mut self, name: impl ToString, data: T) -> Self {
        self.view_models
            .insert(name.to_string(), ViewModel::new(data, Default::default()));
        self
    }

    pub fn view_model_produce<T: 'static>(
        mut self,
        name: impl ToString,
        producer: impl FnOnce(&mut ViewModelProperties) -> T,
    ) -> Self {
        self.view_models
            .insert(name.to_string(), ViewModel::produce(producer));
        self
    }

    /// Run the app!
    ///
    /// Displays the window and runs the RAUI UI.
    pub fn run_with(
        self,
        mut setup: impl FnMut(&mut Application),
    ) -> Result<(), tetra::TetraError> {
        // Create a new tetra context, setting the window title and size
        tetra::ContextBuilder::new(&self.window_title, self.window_size.0, self.window_size.1)
            // Configure the tetra window options
            .resizable(true)
            .key_repeat(true)
            .show_mouse(true)
            .build()?
            // And run the Tetra game. We pass it a closure that returns our App
            .run(|context| {
                let mut app = App::new(
                    context,
                    self.widget_tree,
                    tetra::graphics::Color::rgba(
                        self.clear_color.r,
                        self.clear_color.g,
                        self.clear_color.b,
                        self.clear_color.a,
                    ),
                    self.on_update,
                    self.on_event,
                    self.on_draw,
                    self.view_models,
                )?;
                setup(&mut app.ui.application);
                Ok(app)
            })?;

        Ok(())
    }

    pub fn run(self) -> Result<(), tetra::TetraError> {
        self.run_with(|_| {})
    }
}

/// Our App struct is responsible for handling the Tetra events and rendering
/// the GUI when necessary
struct App {
    /// The UI field is a `TetraSimpleHost` which comes from the
    /// `raui_tetra_renderer` crate and helps wrap some the of the setup
    /// necessary to use RAUI with Tetra.
    ui: TetraSimpleHost,
    /// Color of the window background.
    clear_color: tetra::graphics::Color,
    /// Tetra on update callback.
    on_update: RauiQuickStartOnUpdate,
    /// Tetra on event callback.
    on_event: RauiQuickStartOnEvent,
    /// Tetra on draw callback.
    on_draw: RauiQuickStartOnDraw,
}

impl App {
    fn new(
        context: &mut tetra::Context,
        tree: WidgetNode,
        clear_color: tetra::graphics::Color,
        on_update: RauiQuickStartOnUpdate,
        on_event: RauiQuickStartOnEvent,
        on_draw: RauiQuickStartOnDraw,
        view_models: ViewModelCollection,
    ) -> Result<Self, tetra::TetraError> {
        // Finally we need to provide a setup function that will initialize the
        // RAUI application created by the TetraSimpleHost. We will use the
        // default setup function provided by RAUI.
        let setup = raui::core::widget::setup;

        // Create a TetraSimpleHost out of all the pieces we assigned above
        let mut ui = TetraSimpleHost::new(context, tree, &[], &[], setup)?;
        ui.application.view_models = view_models;

        Ok(Self {
            ui,
            clear_color,
            on_update,
            on_event,
            on_draw,
        })
    }
}

/// Finally, we need to hook into the Tetra events and drive the RAUI UI. For
/// each of these functions we just forward the arguments to the our
/// `TetraSimpleHost` and let it paint the UI on the screen.
impl tetra::State for App {
    fn update(&mut self, ctx: &mut tetra::Context) -> Result<(), tetra::TetraError> {
        if let Some(callback) = &mut self.on_update {
            if callback(ctx, &mut self.ui) {
                self.ui.application.mark_dirty();
            }
        }
        self.ui.update(ctx);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut tetra::Context) -> Result<(), tetra::TetraError> {
        // Clear the screen white first
        tetra::graphics::clear(ctx, self.clear_color);
        if let Some(callback) = &mut self.on_draw {
            if callback(ctx, &mut self.ui) {
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
            if callback(ctx, &mut self.ui, event.clone()) {
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
