pub mod declarative;

use crate::{
    asset_manager::AssetsManager, interactions::AppInteractionsEngine, TesselateToGraphics, Vertex,
};
use glutin::event::{Event, WindowEvent};
use raui_core::{
    application::Application,
    layout::{default_layout_engine::DefaultLayoutEngine, CoordsMapping},
    widget::utils::{Color, Rect},
};
use raui_tesselate_renderer::TesselateRenderer;
use spitfire_fontdue::TextRenderer;
use spitfire_glow::{
    graphics::{Graphics, GraphicsBatch, Shader, Texture},
    renderer::{GlowTextureFormat, GlowUniformValue},
};
use std::{collections::HashMap, time::Instant};

pub use spitfire_glow::app::{App, AppConfig};

macro_rules! hash_map {
    ($($key:ident => $value:expr),* $(,)?) => {{
        let mut result = HashMap::default();
        $(
            result.insert(stringify!($key).into(), $value);
        )*
        result
    }};
}

pub(crate) struct SharedApp {
    #[allow(clippy::type_complexity)]
    on_update: Option<Box<dyn FnMut(&mut Application)>>,
    /// fn(delta time, graphics interface)
    #[allow(clippy::type_complexity)]
    on_redraw: Option<Box<dyn FnMut(f32, &mut Graphics<Vertex>)>>,
    #[allow(clippy::type_complexity)]
    on_event: Option<Box<dyn FnMut(&mut Application, Event<()>) -> bool>>,
    application: Application,
    interactions: AppInteractionsEngine,
    timer: Instant,
    assets: AssetsManager,
    coords_mapping: CoordsMapping,
    missing_texutre: Option<Texture>,
    glyphs_texture: Option<Texture>,
    colored_shader: Option<Shader>,
    textured_shader: Option<Shader>,
    text_shader: Option<Shader>,
}

impl Default for SharedApp {
    fn default() -> Self {
        let mut application = Application::default();
        application.setup(raui_core::widget::setup);
        application.setup(raui_material::setup);
        Self {
            on_update: None,
            on_redraw: None,
            on_event: None,
            application,
            interactions: Default::default(),
            timer: Instant::now(),
            assets: Default::default(),
            coords_mapping: Default::default(),
            missing_texutre: None,
            glyphs_texture: None,
            colored_shader: None,
            textured_shader: None,
            text_shader: None,
        }
    }
}

impl SharedApp {
    fn init(&mut self, graphics: &mut Graphics<Vertex>) {
        self.missing_texutre = Some(graphics.pixel_texture([255, 255, 255]).unwrap());
        self.glyphs_texture = Some(graphics.pixel_texture([0, 0, 0]).unwrap());
        self.colored_shader = Some(
            graphics
                .shader(Shader::COLORED_VERTEX_2D, Shader::PASS_FRAGMENT)
                .unwrap(),
        );
        self.textured_shader = Some(
            graphics
                .shader(Shader::TEXTURED_VERTEX_2D, Shader::TEXTURED_FRAGMENT)
                .unwrap(),
        );
        self.text_shader = Some(
            graphics
                .shader(Shader::TEXT_VERTEX, Shader::TEXT_FRAGMENT)
                .unwrap(),
        );
    }

    fn redraw(&mut self, graphics: &mut Graphics<Vertex>) {
        let elapsed = self.timer.elapsed();
        self.timer = Instant::now();
        if let Some(callback) = self.on_update.as_mut() {
            callback(&mut self.application);
        }
        if let Some(callback) = self.on_redraw.as_mut() {
            callback(elapsed.as_secs_f32(), graphics);
        }
        self.assets.maintain();
        self.application.animations_delta_time = elapsed.as_secs_f32();
        self.coords_mapping = CoordsMapping::new(Rect {
            left: 0.0,
            right: graphics.main_camera.viewport_size.x,
            top: 0.0,
            bottom: graphics.main_camera.viewport_size.y,
        });
        if self.application.process() {
            let _ = self
                .application
                .layout(&self.coords_mapping, &mut DefaultLayoutEngine);
        }
        self.application.interact(&mut self.interactions).unwrap();
        self.application.consume_signals();
        self.assets.load(self.application.rendered_tree(), graphics);
        let matrix = graphics.main_camera.projection_matrix().into_col_array();
        graphics.stream.batch_end();
        for shader in [
            self.colored_shader.clone(),
            self.textured_shader.clone(),
            self.text_shader.clone(),
        ] {
            graphics.stream.batch(GraphicsBatch {
                shader,
                uniforms: hash_map! {
                    u_image => GlowUniformValue::I1(0),
                    u_projection_view => GlowUniformValue::M4(matrix),
                },
                ..Default::default()
            });
            graphics.stream.batch_end();
        }
        let mut text_renderer = TextRenderer::<Color>::new(1024, 1024);
        let mut converter = TesselateToGraphics {
            colored_shader: self.colored_shader.as_ref().unwrap(),
            textured_shader: self.textured_shader.as_ref().unwrap(),
            text_shader: self.text_shader.as_ref().unwrap(),
            glyphs_texture: self.glyphs_texture.as_ref().unwrap(),
            missing_texture: self.missing_texutre.as_ref().unwrap(),
            assets: &self.assets,
            clip_stack: Vec::with_capacity(64),
            viewport_height: graphics.main_camera.viewport_size.y as _,
        };
        let mut renderer = TesselateRenderer::new(
            &self.assets,
            &mut converter,
            &mut graphics.stream,
            &mut text_renderer,
        );
        let _ = self.application.render(&self.coords_mapping, &mut renderer);
        let (image, [w, h, d]) = text_renderer.into_image();
        self.glyphs_texture.as_mut().unwrap().upload(
            w as _,
            h as _,
            d as _,
            GlowTextureFormat::Luminance,
            &image,
        );
    }

    fn event(&mut self, event: Event<()>) -> bool {
        self.interactions.event(&event, &self.coords_mapping);
        if let Event::WindowEvent {
            event: WindowEvent::Resized(_),
            ..
        } = &event
        {
            self.application.mark_dirty();
        }
        self.on_event
            .as_mut()
            .map(|callback| callback(&mut self.application, event))
            .unwrap_or(true)
    }
}
