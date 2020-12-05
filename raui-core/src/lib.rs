pub mod application;
pub mod messenger;
#[macro_use]
pub mod props;
pub mod renderer;
pub mod state;
#[macro_use]
pub mod widget;
pub mod signals;

pub type Scalar = f32;

pub mod prelude {
    pub use crate::{
        application::*,
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
            unit::{content::*, image::*, list::*, text::*},
        },
        Scalar,
    };
}
