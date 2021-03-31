pub mod renderer;
pub mod tesselation;

use raui_core::widget::{unit::image::ImageBoxMaterial, WidgetId};

#[cfg(feature = "index32")]
pub type Index = u32;
#[cfg(not(feature = "index32"))]
pub type Index = u16;

#[derive(Debug, Clone)]
pub enum Error {
    WidgetHasNoLayout(WidgetId),
    UnsupportedImageMaterial(ImageBoxMaterial),
    CouldNotTesselateText(WidgetId),
}

pub mod prelude {
    pub use crate::{renderer::*, tesselation::*};
}
