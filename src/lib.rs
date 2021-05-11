#[cfg(test)]
mod tests;

pub mod core {
    pub use raui_core::*;
}

pub mod renderer {
    #[cfg(feature = "material")]
    pub mod material {
        pub use raui_material::*;
    }

    #[cfg(feature = "binary")]
    pub mod binary {
        pub use raui_binary_renderer::*;
    }
    #[cfg(feature = "html")]
    pub mod html {
        pub use raui_html_renderer::*;
    }
    #[cfg(feature = "json")]
    pub mod json {
        pub use raui_json_renderer::*;
    }
    #[cfg(feature = "ron")]
    pub mod ron {
        pub use raui_ron_renderer::*;
    }
    #[cfg(feature = "tesselate")]
    pub mod tesselate {
        pub use raui_tesselate_renderer::*;
    }
    #[cfg(feature = "yaml")]
    pub mod yaml {
        pub use raui_yaml_renderer::*;
    }
}

pub mod prelude {
    #[cfg(feature = "material")]
    pub use raui_material::prelude::*;

    pub use raui_core::prelude::*;
}
