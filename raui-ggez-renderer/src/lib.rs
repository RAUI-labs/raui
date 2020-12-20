pub mod interactive;
pub mod renderer;
pub mod resources;

use raui_core::widget::{
    unit::{image::ImageBoxMaterial, WidgetUnit},
    WidgetId,
};

#[derive(Debug, Clone)]
pub enum Error {
    CouldNotDrawImage(WidgetId),
    CouldNotBuildImageMesh(WidgetId),
    ImageResourceNotFound(WidgetId, String),
    WidgetHasNoLayout(WidgetId),
    UnsupportedImageMaterial(ImageBoxMaterial),
    UnsupportedWidget(WidgetUnit),
}

pub mod prelude {
    pub use crate::{interactive::*, renderer::*, resources::*};
}
