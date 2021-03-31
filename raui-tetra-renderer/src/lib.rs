pub mod interactive;
pub mod renderer;
pub mod resources;
pub mod simple_host;

use raui_tesselate_renderer::Error as TesselationError;

#[derive(Debug, Clone)]
pub enum Error {
    ImageResourceNotFound(String),
    FontResourceNotFound(String),
    Tesselation(TesselationError),
    CannotCreateVertexBuffer,
    CannotCreateIndexBuffer,
    CannotUnpackVertices,
}

impl From<TesselationError> for Error {
    fn from(error: TesselationError) -> Self {
        Self::Tesselation(error)
    }
}

impl From<Error> for tetra::error::TetraError {
    fn from(error: Error) -> Self {
        Self::PlatformError(format!("RAUI Tetra renderer error: {:?}", error))
    }
}

pub mod prelude {
    pub use crate::{interactive::*, renderer::*, resources::*, simple_host::*};
}
