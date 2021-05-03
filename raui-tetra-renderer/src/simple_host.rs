use crate::{
    interactive::TetraInteractionsEngine, renderer::TetraRenderer, resources::TetraResources,
};
use raui_core::{
    application::Application,
    layout::{default_layout_engine::DefaultLayoutEngine, CoordsMapping, CoordsMappingScaling},
    signals::Signal,
    widget::{node::WidgetNode, utils::Rect},
    Scalar,
};
use tetra::{
    graphics::{text::Font, Texture},
    time, window, Context, Event,
};

pub struct TetraSimpleHost {
    pub application: Application,
    pub resources: TetraResources,
    pub interactions: TetraInteractionsEngine,
    pub scaling: CoordsMappingScaling,
}

impl TetraSimpleHost {
    /// F: (id, font size, font scale, path)
    /// T: (id, path)
    pub fn new<'a, F, T, S>(
        context: &mut Context,
        tree: WidgetNode,
        fonts: F,
        textures: T,
        setup: S,
    ) -> tetra::Result<Self>
    where
        F: IntoIterator<Item = &'a (&'a str, usize, Scalar, &'a str)>,
        T: IntoIterator<Item = &'a (&'a str, &'a str)>,
        S: FnMut(&mut Application),
    {
        let mut resources = TetraResources::default();
        for (id, size, scale, path) in fonts.into_iter() {
            resources.fonts.insert(
                format!("{}:{}", id, size),
                (
                    *scale,
                    Font::vector(context, path, *size as Scalar * *scale)?,
                ),
            );
        }
        for (id, path) in textures.into_iter() {
            resources
                .textures
                .insert(id.to_string(), Texture::new(context, path)?);
        }

        let mut application = Application::new();
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

    pub fn draw(&mut self, context: &mut Context) -> tetra::Result {
        let mapping = self.make_coords_mapping(context);
        let mut renderer = TetraRenderer::new(context, &mut self.resources);
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
