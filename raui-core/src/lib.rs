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
pub mod data_binding;
pub mod interactive;
pub mod layout;
pub mod signals;

#[cfg(feature = "scalar64")]
pub type Scalar = f64;
#[cfg(not(feature = "scalar64"))]
pub type Scalar = f32;
#[cfg(feature = "integer64")]
pub type Integer = i64;
#[cfg(not(feature = "integer64"))]
pub type Integer = i32;

pub use raui_derive::*;
use serde::{de::DeserializeOwned, Serialize};
pub use serde_yaml::{Number as PrefabNumber, Value as PrefabValue};

#[derive(Debug, Clone)]
pub enum PrefabError {
    CouldNotSerialize(String),
    CouldNotDeserialize(String),
}

pub trait Prefab: Serialize + DeserializeOwned {
    fn from_prefab(data: PrefabValue) -> Result<Self, PrefabError> {
        match serde_yaml::from_value(data) {
            Ok(result) => Ok(result),
            Err(error) => Err(PrefabError::CouldNotDeserialize(error.to_string())),
        }
    }

    fn to_prefab(&self) -> Result<PrefabValue, PrefabError> {
        match serde_yaml::to_value(self) {
            Ok(result) => Ok(result),
            Err(error) => Err(PrefabError::CouldNotSerialize(error.to_string())),
        }
    }
}

pub mod prelude {
    pub use crate::{
        animator::*,
        application::*,
        data_binding::*,
        destruct, implement_message_data, implement_props_data,
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
        unpack_named_slots, widget,
        widget::*,
        widget::{
            component::*,
            component::{
                containers::{
                    content_box::*, flex_box::*, grid_box::*, horizontal_box::*, portal_box::*,
                    scroll_box::*, size_box::*, switch_box::*, variant_box::*, vertical_box::*,
                    wrap_box::*,
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
            unit::{area::*, content::*, flex::*, grid::*, image::*, size::*, text::*},
            utils::*,
        },
        widget_wrap, Integer, MessageData, Prefab, PrefabError, PropsData, Scalar,
    };
}
