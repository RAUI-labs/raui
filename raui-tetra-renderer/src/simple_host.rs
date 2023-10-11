use crate::{
    interactive::TetraInteractionsEngine, renderer::TetraRenderer, resources::TetraResources,
};
use raui_core::{
    application::Application,
    layout::{default_layout_engine::DefaultLayoutEngine, CoordsMapping, CoordsMappingScaling},
    signals::Signal,
    widget::{node::WidgetNode, utils::Rect},
    Logger, Scalar,
};
use tetra::{
    graphics::{text::Font, Texture},
    time, window, Context, Event,
};

/// A host that manages a RAUI application in a Tetra game
pub struct TetraSimpleHost {
    pub application: Application,
    pub resources: TetraResources,
    pub interactions: TetraInteractionsEngine,
    pub scaling: CoordsMappingScaling,
}

/// A font that will be pre-loaded during the init of a [`TetraSimpleHost`]
pub struct PreloadedFont<'a> {
    /// The ID that will be used to reference the font in UI code
    pub id: &'a str,
    /// The size to load the font at
    ///
    /// Tetra will cache the font on the GPU, rendered at the given size
    pub size: usize,
    /// An additional scale to apply when rendering the font
    pub scale: f32,
    /// The path to the font file
    pub path: &'a str,
}

/// A texture that will be pre-loaded during the init of a [`TetraSimpleHost`]
pub struct PreloadedTexture<'a> {
    /// The ID that will be used to reference the texture in UI code
    pub id: &'a str,
    /// The path to the texture image file
    pub path: &'a str,
}

impl TetraSimpleHost {
    /// Create a new [`TetraSimpleHost`]
    ///
    /// # Preloading Textures and Fonts
    ///
    /// The `preload_fonts` and `preload_textures` parameters may be used to instruct the host to
    /// load the given textures and fonts now before starting the app. Textures and fonts **do not**
    /// have to be pre-loaded, in which case you would simply pass an empty slice ( `&[]` ) for each
    /// argument.
    ///
    /// If you do not wish to pre-load the textures and fonts, you may simply supply the path to the
    /// file when specifying images and fonts in the UI code and they will be loaded on-demand.
    pub fn new<'a, F, T, S>(
        context: &mut Context,
        tree: WidgetNode,
        preload_fonts: F,
        preload_textures: T,
        setup: S,
    ) -> tetra::Result<Self>
    where
        F: IntoIterator<Item = &'a PreloadedFont<'a>>,
        T: IntoIterator<Item = &'a PreloadedTexture<'a>>,
        S: FnMut(&mut Application),
    {
        let mut resources = TetraResources::default();
        for font in preload_fonts.into_iter() {
            resources.fonts.insert(
                format!("{}:{}", font.id, font.size),
                (
                    font.scale,
                    Font::vector(context, font.path, font.size as Scalar * font.scale)?,
                ),
            );
        }
        for texture in preload_textures.into_iter() {
            resources
                .textures
                .insert(texture.id.to_string(), Texture::new(context, texture.path)?);
        }

        let mut application = Application::default();
        application.setup(setup);
        application.apply(tree);

        let mut interactions = TetraInteractionsEngine::default();
        interactions.engine.deselect_when_no_button_found = true;

        Ok(Self {
            application,
            resources,
            interactions,
            scaling: Default::default(),
        })
    }

    pub fn update(&mut self, context: &mut Context) -> Vec<Signal> {
        self.interactions.update(context);
        self.application.animations_delta_time = time::get_delta_time(context).as_secs_f32();
        if self.application.process() {
            let mapping = self.make_coords_mapping(context);
            let _ = self.application.layout(&mapping, &mut DefaultLayoutEngine);
        }
        self.application.interact(&mut self.interactions).unwrap();
        self.application.consume_signals()
    }

    pub fn draw<L>(&mut self, context: &mut Context, logger: L) -> tetra::Result
    where
        L: Logger,
    {
        let mapping = self.make_coords_mapping(context);
        let mut renderer = TetraRenderer::new(context, &mut self.resources, logger);
        self.application.render(&mapping, &mut renderer)?;
        Ok(())
    }

    pub fn event(&mut self, context: &mut Context, event: &Event) {
        if let Event::Resized { .. } = event {
            self.application.mark_dirty();
        }
        let mapping = self.make_coords_mapping(context);
        self.interactions.event(context, event, &mapping);
    }

    fn make_coords_mapping(&self, context: &Context) -> CoordsMapping {
        let (width, height) = window::get_size(context);
        let area = Rect {
            left: 0.0,
            right: width as Scalar,
            top: 0.0,
            bottom: height as Scalar,
        };
        CoordsMapping::new_scaling(area, self.scaling)
    }
}
