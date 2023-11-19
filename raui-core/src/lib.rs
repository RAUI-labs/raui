//! RAUI core types and components
//!
//! The things that most users will be interested in here are the [components][widget::component].
//! Those have more documentation on how to use widgets, components, etc. in your app.

pub mod application;
#[macro_use]
pub mod messenger;
#[macro_use]
pub mod props;
pub mod renderer;
pub mod state;
#[macro_use]
pub mod widget;
pub mod animator;
pub mod interactive;
pub mod layout;
pub mod signals;
pub mod tester;
pub mod view_model;

pub type Scalar = f32;
pub type Integer = i32;
pub type UnsignedInteger = u32;

pub use raui_derive::*;
use serde::{de::DeserializeOwned, Serialize};

#[doc(inline)]
pub use serde_json::{Number as PrefabNumber, Value as PrefabValue};

pub use intuicio_data::{lifetime::*, managed::*, type_hash::*};

#[doc(hidden)]
pub mod prelude {
    pub use crate::{
        animator::*,
        application::*,
        implement_message_data, implement_props_data,
        interactive::default_interactions_engine::*,
        interactive::*,
        layout::default_layout_engine::*,
        layout::*,
        make_widget,
        messenger::*,
        post_hooks, pre_hooks,
        props::*,
        renderer::*,
        signals::*,
        state::*,
        unpack_named_slots,
        view_model::*,
        widget,
        widget::*,
        widget::{
            component::*,
            component::{
                containers::{
                    anchor_box::*, area_box::*, content_box::*, context_box::*, flex_box::*,
                    grid_box::*, hidden_box::*, horizontal_box::*, portal_box::*, scroll_box::*,
                    size_box::*, switch_box::*, tabs_box::*, tooltip_box::*, variant_box::*,
                    vertical_box::*, wrap_box::*,
                },
                image_box::*,
                interactive::*,
                interactive::{button::*, input_field::*, navigation::*, scroll_view::*},
                space_box::*,
                text_box::*,
            },
            context::*,
            node::*,
            unit::*,
            unit::{area::*, content::*, flex::*, grid::*, image::*, portal::*, size::*, text::*},
            utils::*,
        },
        Integer, LogKind, Logger, MessageData, Prefab, PrefabError, PrintLogger, PropsData, Scalar,
    };
    pub use intuicio_data::{lifetime::*, managed::*, type_hash::*};
}

/// An error that can occur while processing a [`Prefab`]
#[derive(Debug, Clone)]
pub enum PrefabError {
    CouldNotSerialize(String),
    CouldNotDeserialize(String),
}

/// The [`Prefab`] trait is implemented for types that are able to translate to and from
/// [`PrefabValue`]'s
///
/// [`PrefabValue`]'s can then, in turn, be serialized or deserialized for persistance, transfer, or
/// other purposes.
pub trait Prefab: Serialize + DeserializeOwned {
    fn from_prefab(data: PrefabValue) -> Result<Self, PrefabError> {
        match serde_json::from_value(data) {
            Ok(result) => Ok(result),
            Err(error) => Err(PrefabError::CouldNotDeserialize(error.to_string())),
        }
    }

    fn to_prefab(&self) -> Result<PrefabValue, PrefabError> {
        match serde_json::to_value(self) {
            Ok(result) => Ok(result),
            Err(error) => Err(PrefabError::CouldNotSerialize(error.to_string())),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LogKind {
    Info,
    Warning,
    Error,
}

/// Common logging interface that custom log engines should follow to enable their reusability
/// across different modules that will log messages to text output targets.
/// Objects that implement this trait should be considered text output targets, for example text
/// streams, terminal, network-based loggers, even application screen.
pub trait Logger {
    /// Log message to this type of text output target.
    ///
    /// # Arguments
    /// * `kind` - Kind of log message.
    /// * `message` - Message string slice.
    fn log(&mut self, _kind: LogKind, _message: &str) {}
}

impl Logger for () {}

/// Prints log messages to terminal via println! macro.
pub struct PrintLogger;

impl Logger for PrintLogger {
    fn log(&mut self, kind: LogKind, message: &str) {
        println!("{:?} | {}", kind, message);
    }
}
