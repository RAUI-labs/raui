pub mod application;
pub mod messenger;
#[macro_use]
pub mod props;
pub mod renderer;
pub mod state;
#[macro_use]
pub mod widget;
pub mod layout;
pub mod signals;

pub type Scalar = f32;
pub type Integer = i32;

pub mod prelude {
    pub use crate::{
        application::*,
        layout::default_layout_engine::*,
        layout::*,
        messenger::*,
        props::*,
        renderer::*,
        signals::*,
        state::*,
        widget::*,
        widget::{
            component::*,
            context::*,
            node::*,
            unit::*,
            unit::{content::*, flex::*, grid::*, image::*, size::*, text::*},
            utils::*,
        },
        Integer, Scalar,
    };
}
