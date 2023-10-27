//! RAUI is a renderer agnostic UI system that is heavily inspired by **React**'s declarative UI
//! composition and the **UE4 Slate** widget components system.
//!
//! > 🗣 **Pronunciation:** RAUI is pronounced like **"ra"** ( the Egyptian god ) + **"oui"**
//! > (french for "yes" ) — [Audio Example][pronounciation].
//!
//! [pronounciation]: https://itinerarium.github.io/phoneme-synthesis/?w=/%27rawi/
//!
//! The main idea behind RAUI architecture is to treat UI as another data source that you transform
//! into your target renderable data format used by your rendering engine of choice.
//!
//! # Architecture
//!
//! ## [`Application`]
//!
//! [`Application`] is the central point of user interest. It performs whole UI processing logic.
//! There you apply widget tree that wil be processed, send messages from host application to
//! widgets and receive signals sent from widgets to host application.
//!
//! [`Application`]: core::application::Application
//!
//! ```rust
//! # use raui::prelude::*;
//! # use raui::renderer::json::JsonRenderer;
//! # fn app(context: WidgetContext) -> WidgetNode {
//! #     let WidgetContext { named_slots, .. } = context;
//! #     unpack_named_slots!(named_slots => { title, content });
//! #     make_widget!(nav_vertical_box)
//! #         .listed_slot(title)
//! #         .listed_slot(content)
//! #         .into()
//! # }
//! # fn title_bar(context: WidgetContext) -> WidgetNode {
//! #     let WidgetContext { props, .. } = context;  
//! #     make_widget!(text_box)
//! #         .with_props(TextBoxProps {
//! #             text: props.read_cloned_or_default::<String>(),
//! #             font: TextBoxFont {
//! #                 name: "verdana.ttf".to_owned(),
//! #                 size: 64.0,
//! #             },
//! #             color: Color {
//! #                 r: 0.0,
//! #                 g: 0.0,
//! #                 b: 0.5,
//! #                 a: 1.0,
//! #             },
//! #             horizontal_align: TextBoxHorizontalAlign::Center,
//! #             ..Default::default()
//! #         })
//! #         .into()
//! # }
//! # fn text_button(context: WidgetContext) -> WidgetNode {
//! #     let WidgetContext { props, .. } = context;
//! #     make_widget!(button)
//! #         .named_slot("content", make_widget!(text_box)
//! #             .with_props(TextBoxProps {
//! #                 text: props.read_cloned_or_default::<String>(),
//! #                 font: TextBoxFont {
//! #                     name: "verdana.ttf".to_owned(),
//! #                     size: 32.0,
//! #                 },
//! #                 color: Color {
//! #                     r: 1.0,
//! #                     g: 0.0,
//! #                     b: 0.5,
//! #                     a: 1.0,
//! #                 },
//! #                 ..Default::default()
//! #             }))
//! #             .into()
//! # }
//! // Coords mapping tell RAUI renderers how to convert coordinates
//! // between virtual-space and ui-space.
//! let mapping = CoordsMapping::new(Rect {
//!     left: 0.0,
//!     right: 1024.0,
//!     top: 0.0,
//!     bottom: 576.0,
//! });
//!
//! // Application is UI host.
//! let mut application = Application::default();
//! // we use setup functions to register component and props mappings for serialization.
//! application.setup(setup);
//! // we can also register them at any time one by one.
//! application.register_component("app", FnWidget::pointer(app));
//!
//! // Widget tree is simply a set of nested widget nodes.
//! let tree = make_widget!(app)
//!     .named_slot("title", make_widget!(title_bar).with_props("Hello".to_owned()))
//!     .named_slot("content", make_widget!(vertical_box)
//!         .listed_slot(make_widget!(text_button).key("hi").with_props("Say hi!".to_owned()))
//!         .listed_slot(make_widget!(text_button).key("exit").with_props("Exit!".to_owned()))
//!     );
//!
//! // some dummy widget tree renderer.
//! // it reads widget unit tree and transforms it into target format.
//! let mut renderer = JsonRenderer::default();
//!
//! // `apply()` sets new widget tree.
//! application.apply(tree);
//!
//! // `render()` calls renderer to perform transformations on processed application widget tree.
//! // by default application won't process widget tree if nothing was changed.
//! // "change" is either any widget state change, or new message sent to any widget (messages
//! // can be sent from application host, for example a mouse click, or from another widget).
//! application.forced_process();
//! if let Ok(output) = application.render::<JsonRenderer, String, _>(&mapping, &mut renderer) {
//!     println!("* OUTPUT:\n{}", output);
//! }
//! ```
//!
//! ## Widgets
//!
//! Widgets are divided into three categories:
//! - **[`WidgetNode`]** - used as source UI trees (variant that can be either a component, unit or
//!   none)
//!
//! [`WidgetNode`]: core::widget::node::WidgetNode
//!
//! ```rust
//! # use raui::prelude::*;
//! # fn app(context: WidgetContext) -> WidgetNode {
//! #     let WidgetContext { named_slots, .. } = context;
//! #     unpack_named_slots!(named_slots => { title, content });
//! #     make_widget!(nav_vertical_box)
//! #         .listed_slot(title)
//! #         .listed_slot(content)
//! #         .into()
//! # }
//! # fn title_bar(context: WidgetContext) -> WidgetNode {
//! #     let WidgetContext { props, .. } = context;
//! #     make_widget!(text_box)
//! #         .with_props(TextBoxProps {
//! #             text: props.read_cloned_or_default::<String>(),
//! #             font: TextBoxFont {
//! #                 name: "verdana.ttf".to_owned(),
//! #                 size: 64.0,
//! #             },
//! #             color: Color {
//! #                 r: 0.0,
//! #                 g: 0.0,
//! #                 b: 0.5,
//! #                 a: 1.0,
//! #             },
//! #             horizontal_align: TextBoxHorizontalAlign::Center,
//! #             ..Default::default()
//! #         })
//! #         .into()
//! # }
//! # fn text_button(context: WidgetContext) -> WidgetNode {
//! #     let WidgetContext { props, .. } = context;
//! #     make_widget!(button)
//! #         .named_slot("content", make_widget!(text_box)
//! #             .with_props(TextBoxProps {
//! #                 text: props.read_cloned_or_default::<String>(),
//! #                 font: TextBoxFont {
//! #                     name: "verdana.ttf".to_owned(),
//! #                     size: 32.0,
//! #                 },
//! #                 color: Color {
//! #                     r: 1.0,
//! #                     g: 0.0,
//! #                     b: 0.5,
//! #                     a: 1.0,
//! #                 },
//! #                 ..Default::default()
//! #             }))
//! #             .into()
//! # }
//! let tree = make_widget!(app)
//!     .named_slot("title", make_widget!(title_bar).with_props("Hello".to_owned()))
//!     .named_slot("content", make_widget!(vertical_box)
//!         .listed_slot(make_widget!(text_button).key("hi").with_props("Say hi!".to_owned()))
//!         .listed_slot(make_widget!(text_button).key("exit").with_props("Exit!".to_owned()))
//!     );
//! ```
//!
//! - **[`WidgetComponent`]** - you can think of them as Virtual DOM nodes, they store:
//!   - pointer to _component function_ (that process their data)
//!   - unique _key_ (that is a part of widget ID and will be used to tell the system if it should
//!     carry its _state_ to next processing run)
//!   - boxed cloneable _properties_ data
//!   - _listed slots_ (simply: widget children)
//!   - _named slots_ (similar to listed slots: widget children, but these ones have names assigned
//!     to them, so you can access them by name instead of by index)
//! - **[`WidgetUnit`]** - an atomic element that renderers use to convert into target renderable
//!   data format for rendering engine of choice.
//!   ```rust
//!   # use raui::prelude::*;
//!   TextBoxNode {
//!       text: "Hello World".to_owned(),
//!       ..Default::default()
//!   };
//!   ```
//!
//! [`WidgetComponent`]: core::widget::component::WidgetComponent
//!
//! [`WidgetUnit`]: core::widget::unit::WidgetUnit
//!
//! ## Component Function
//!
//! Component functions are static functions that transforms input data (properties, state or
//! neither of them) into output widget tree (usually used to simply wrap another components tree
//! under one simple component, where at some point the simplest components returns final
//! _[`WidgetUnit`]'s_). They work together as a chain of transforms - root component applies some
//! properties into children components using data from its own properties or state.
//!
//! ```rust
//! # use raui::prelude::*;
//! # use serde::{Serialize, Deserialize};
//! #[derive(PropsData, Debug, Default, Copy, Clone, Serialize, Deserialize)]
//! struct AppProps {
//!     #[serde(default)]
//!     pub index: usize,
//! }
//! fn app(context: WidgetContext) -> WidgetNode {
//!     let WidgetContext {
//!         props, named_slots, ..
//!     } = context;
//!     // easy way to get widgets from named slots.
//!     unpack_named_slots!(named_slots => { title, content });
//!     let index = props.read::<AppProps>().map(|p| p.index).unwrap_or(0);
//!
//!     // we always return new widgets tree.
//!     make_widget!(vertical_box)
//!         .key(index)
//!         .listed_slot(title)
//!         .listed_slot(content)
//!         .into()
//! }
//! ```
//! ### States
//!
//! This may bring up a question: _**"If i use only functions and no objects to tell how to
//! visualize UI, how do i keep some data between each render run?"**_. For that you use _states_.
//! State is a data that is stored between each processing calls as long as given widget is alive
//! (that means: as long as widget id stays the same between two processing calls, to make sure your
//! widget stays the same, you use keys - if no key is assigned, system will generate one for your
//! widget but that will make it possible to die at any time if for example number of widget
//! children changes in your common parent, your widget will change its id when key wasn't
//! assigned). Some additional notes: While you use _properties_ to send information down the tree
//! and _states_ to store widget data between processing cals, you can communicate with another
//! widgets and host application using messages and signals! More than that, you can use hooks to
//! listen for widget life cycle and perform actions there. It's worth noting that state uses
//! _properties_ to hold its data, so by that you can for example attach multiple hooks that each of
//! them uses different data type as widget state, this opens the doors to be very creative when
//! combining different hooks that operate on the same widget.
//! ```rust
//! # use raui::prelude::*;
//! # use serde::{Serialize, Deserialize};
//! #[derive(PropsData, Debug, Default, Copy, Clone, Serialize, Deserialize)]
//! struct ButtonState {
//!     #[serde(default)]
//!     pub pressed: bool,
//! }
//! ```
//!
//! ## Hooks
//!
//! Hooks are used to put common widget logic into separate functions that can be chained in widgets
//! and another hooks (you can build a reusable dependency chain of logic with that). Usually it is
//! used to listen for life cycle events such as mount, change and unmount, additionally you can
//! chain hooks to be processed sequentially in order they are chained in widgets and other hooks.
//!
//! ```rust
//! # use raui::prelude::*;
//! # use serde::{Serialize, Deserialize};
//! # #[derive(PropsData, Debug, Copy, Clone, Serialize, Deserialize)]
//! # struct ButtonState {
//! #     pressed: bool
//! # }
//! #[derive(MessageData, Debug, Copy, Clone, PartialEq, Eq)]
//! enum ButtonAction {
//!     Pressed,
//!     Released,
//! }
//!
//! fn use_empty(context: &mut WidgetContext) {
//!     context.life_cycle.mount(|_| {
//!         println!("* EMPTY MOUNTED");
//!     });
//!
//!     context.life_cycle.change(|_| {
//!         println!("* EMPTY CHANGED");
//!     });
//!
//!     context.life_cycle.unmount(|_| {
//!         println!("* EMPTY UNMOUNTED");
//!     });
//! }
//!
//! // you use life cycle hooks for storing closures that will be called when widget will be
//! // mounted/changed/unmounted. they exists for you to be able to reuse some common logic across
//! // multiple components. each closure provides arguments such as:
//! // - widget id
//! // - widget state
//! // - message sender (this one is used to message other widgets you know about)
//! // - signal sender (this one is used to message application host)
//! // although this hook uses only life cycle, you can make different hooks that use many
//! // arguments, even use context you got from the component!
//! #[pre_hooks(use_empty)]
//! fn use_button(context: &mut WidgetContext) {
//!     context.life_cycle.mount(|context| {
//!         println!("* BUTTON MOUNTED: {}", context.id.key());
//!         let _ = context.state.write(ButtonState { pressed: false });
//!     });
//!
//!     context.life_cycle.change(|context| {
//!         println!("* BUTTON CHANGED: {}", context.id.key());
//!         for msg in context.messenger.messages {
//!             if let Some(msg) = msg.as_any().downcast_ref::<ButtonAction>() {
//!                 let pressed = match msg {
//!                     ButtonAction::Pressed => true,
//!                     ButtonAction::Released => false,
//!                 };
//!                 println!("* BUTTON ACTION: {:?}", msg);
//!                 let _ = context.state.write(ButtonState { pressed });
//!                 let _ = context.signals.write(*msg);
//!             }
//!         }
//!     });
//!
//!     context.life_cycle.unmount(|context| {
//!         println!("* BUTTON UNMOUNTED: {}", context.id.key());
//!     });
//! }
//!
//! #[pre_hooks(use_button)]
//! fn button(mut context: WidgetContext) -> WidgetNode {
//!     let WidgetContext { key, props, .. } = context;
//!     println!("* PROCESS BUTTON: {}", key);
//!
//!     make_widget!(text_box).key(key).merge_props(props.clone()).into()
//! }
//! ```
//!
//! What happens under the hood:
//! - Application calls `button` on a node
//!     - `button` calls `use_button` hook
//!         - `use_button` calls `use_empty` hook
//!     - `use_button` logic is executed
//! - `button` logic is executed
//!
//! ## Layouting
//!
//! RAUI exposes the [`Application::layout()`][core::application::Application::layout] API to allow
//! use of virtual-to-real coords mapping and custom layout engines to perform widget tree
//! positioning data, which is later used by custom UI renderers to specify boxes where given
//! widgets should be placed. Every call to perform layouting will store a layout data inside
//! Application, you can always access that data at any time. There is a [`DefaultLayoutEngine`]
//! that does this in a generic way. If you find some part of its pipeline working different than
//! what you've expected, feel free to create your custom layout engine!
//!
//! ```rust
//! # use raui::prelude::*;
//! # let tree = WidgetNode::default();
//! # let mapping = CoordsMapping::new(Rect::default());
//! let mut application = Application::default();
//! let mut layout_engine = DefaultLayoutEngine;
//! application.apply(tree);
//! application.forced_process();
//! println!(
//!     "* TREE INSPECTION:\n{:#?}",
//!     application.rendered_tree().inspect()
//! );
//! if application.layout(&mapping, &mut layout_engine).is_ok() {
//!     println!("* LAYOUT:\n{:#?}", application.layout_data());
//! }
//! ```
//!
//! [`DefaultLayoutEngine`]: core::layout::default_layout_engine::DefaultLayoutEngine
//!
//! ## Interactivity
//!
//!
//! RAUI allows you to ease and automate interactions with UI by use of Interactions Engine - this
//! is just a struct that implements [`perform_interactions`] method with reference to Application,
//! and all you should do there is to send user input related messages to widgets. There is
//! [`DefaultInteractionsEngine`] that covers widget navigation, button and input field - actions
//! sent from input devices such as mouse (or any single pointer), keyboard and gamepad. When it
//! comes to UI navigation you can send raw [`NavSignal`] messages to the default interactions
//! engine and despite being able to select/unselect widgets at will, you have typical navigation
//! actions available: up, down, left, right, previous tab/screen, next tab/screen, also being able
//! to focus text inputs and send text input changes to focused input widget. All interactive widget
//! components that are provided by RAUI handle all [`NavSignal`] actions in their hooks, so all
//! user has to do is to just activate navigation features for them (using [`NavItemActive`] unit
//! props). RAUI integrations that want to just use use default interactions engine should make use
//! of this struct composed in them and call its [`interact`] method with information about what
//! input change was made. There is an example of that feature covered in RAUI App crate
//! (`AppInteractionsEngine` struct).
//!
//! [`NavSignal`]: core::widget::component::interactive::navigation::NavSignal
//!
//! [`NavItemActive`]: core::widget::component::interactive::navigation::NavItemActive
//!
//! [`perform_interactions`]: core::interactive::InteractionsEngine::perform_interactions
//!
//! [`interact`]:
//! core::interactive::default_interactions_engine::DefaultInteractionsEngine::interact
//!
//! [`DefaultInteractionsEngine`]:
//! core::interactive::default_interactions_engine::DefaultInteractionsEngine
//!
//! **NOTE: Interactions engines should use layout for pointer events so make sure that you rebuild
//! layout before you perform interactions!**
//!
//! ```rust
//! # use raui::prelude::*;
//! let mut application = Application::default();
//! // default interactions engine covers typical pointer + keyboard + gamepad navigation/interactions.
//! let mut interactions = DefaultInteractionsEngine::default();
//! // we interact with UI by sending interaction messages to the engine.
//! interactions.interact(Interaction::PointerMove(Vec2 { x: 200.0, y: 100.0 }));
//! interactions.interact(Interaction::PointerDown(
//!     PointerButton::Trigger,
//!     Vec2 { x: 200.0, y: 100.0 },
//! ));
//! // navigation/interactions works only if we have navigable items (such as `button`) registered
//! // in some navigable container (usually containers with `nav_` prefix).
//! let tree = make_widget!(nav_content_box)
//!     .key("app")
//!     .listed_slot(make_widget!(button)
//!         .key("button")
//!         .with_props(NavItemActive)
//!         .named_slot("content", make_widget!(image_box).key("icon"))
//!     );
//! application.apply(tree);
//! application.process();
//! let mapping = CoordsMapping::new(Rect {
//!     left: 0.0,
//!     right: 1024.0,
//!     top: 0.0,
//!     bottom: 576.0,
//! });
//! application
//!     .layout(&mapping, &mut DefaultLayoutEngine)
//!     .unwrap();
//! // Since interactions engines require constructed layout to process interactions we have to
//! // process interactions after we layout the UI.
//! application.interact(&mut interactions).unwrap();
//! ```

#[doc(inline)]
pub use raui_core as core;

#[doc(inline)]
#[cfg(feature = "material")]
pub use raui_material as material;

/// Renderer implementations
pub mod renderer {
    #[cfg(feature = "json")]
    pub mod json {
        pub use raui_json_renderer::*;
    }
    #[cfg(feature = "tesselate")]
    pub mod tesselate {
        pub use raui_tesselate_renderer::*;
    }
}

#[doc(hidden)]
pub mod prelude {
    #[cfg(feature = "material")]
    pub use raui_material::prelude::*;

    #[cfg(feature = "retained")]
    pub use raui_retained::*;

    #[cfg(feature = "immediate")]
    pub use raui_immediate::*;

    #[cfg(feature = "immediate-widgets")]
    pub use raui_immediate_widgets::*;

    #[cfg(feature = "json")]
    pub use raui_json_renderer::*;

    #[cfg(feature = "app")]
    pub use raui_app::prelude::*;

    pub use raui_core::prelude::*;
}
