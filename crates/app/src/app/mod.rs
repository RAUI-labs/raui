pub mod declarative;
pub mod immediate;
pub mod retained;

use crate::{
    TesselateToGraphics, Vertex, asset_manager::AssetsManager, interactions::AppInteractionsEngine,
    render_worker::RenderWorkersViewModel, text_measurements::AppTextMeasurementsEngine,
};
use glutin::{
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    window::Window,
};
use raui_core::{
    application::Application,
    interactive::default_interactions_engine::DefaultInteractionsEngine,
    layout::{CoordsMapping, CoordsMappingScaling, default_layout_engine::DefaultLayoutEngine},
    view_model::ViewModel,
    widget::utils::{Color, Rect},
};
use raui_tesselate_renderer::{TesselateRenderer, TessselateRendererDebug};
use spitfire_fontdue::TextRenderer;
use spitfire_glow::{
    app::AppControl,
    graphics::{Graphics, GraphicsBatch, Shader, Texture},
    renderer::{GlowTextureFormat, GlowUniformValue},
};
use std::{collections::HashMap, time::Instant};

pub use spitfire_glow::app::{App, AppConfig};

#[cfg(debug_assertions)]
const DEBUG_VERTEX: &str = r#"#version 300 es
    layout(location = 0) in vec2 a_position;
    out vec4 v_color;
    uniform mat4 u_projection_view;

    void main() {
        gl_Position = u_projection_view * vec4(a_position, 0.0, 1.0);
    }
    "#;

#[cfg(debug_assertions)]
const DEBUG_FRAGMENT: &str = r#"#version 300 es
    precision highp float;
    precision highp int;
    out vec4 o_color;
    uniform float u_time;

    vec3 hsv2rgb(vec3 c) {
        vec4 K = vec4(1.0, 2.0/3.0, 1.0/3.0, 3.0);
        vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
        return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
    }

    void main() {
        vec2 pixel = floor(gl_FragCoord.xy);
        float hue = fract((floor(pixel.x) + floor(pixel.y)) * 0.01 + u_time);
        o_color = vec4(hsv2rgb(vec3(hue, 1.0, 1.0)), 1.0);
    }
    "#;

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
    on_update: Option<Box<dyn FnMut(&mut Application, &mut AppControl)>>,
    /// fn(delta time, graphics interface)
    #[allow(clippy::type_complexity)]
    on_redraw: Option<
        Box<dyn FnMut(f32, &mut Graphics<Vertex>, &mut TextRenderer<Color>, &mut AppControl)>,
    >,
    #[allow(clippy::type_complexity)]
    on_event: Option<
        Box<
            dyn FnMut(
                &mut Application,
                Event<()>,
                &mut Window,
                &mut DefaultInteractionsEngine,
            ) -> bool,
        >,
    >,
    application: Application,
    interactions: AppInteractionsEngine,
    text_renderer: TextRenderer<Color>,
    timer: Instant,
    time: f32,
    assets: AssetsManager,
    coords_mapping: CoordsMapping,
    pub coords_mapping_scaling: CoordsMappingScaling,
    missing_texutre: Option<Texture>,
    glyphs_texture: Option<Texture>,
    colored_shader: Option<Shader>,
    textured_shader: Option<Shader>,
    text_shader: Option<Shader>,
    #[cfg(debug_assertions)]
    debug_shader: Option<Shader>,
    #[cfg(debug_assertions)]
    pub show_raui_aabb_mode: u8,
    #[cfg(debug_assertions)]
    pub show_raui_aabb_key: VirtualKeyCode,
    #[cfg(debug_assertions)]
    pub print_raui_tree_key: VirtualKeyCode,
    #[cfg(debug_assertions)]
    pub print_raui_layout_key: VirtualKeyCode,
    #[cfg(debug_assertions)]
    pub print_raui_interactions_key: VirtualKeyCode,
}

impl Default for SharedApp {
    fn default() -> Self {
        let mut application = Application::default();
        application.setup(raui_core::widget::setup);
        application.setup(raui_material::setup);
        application.view_models.insert(
            RenderWorkersViewModel::VIEW_MODEL.to_owned(),
            ViewModel::new(RenderWorkersViewModel::default(), Default::default()),
        );
        Self {
            on_update: None,
            on_redraw: None,
            on_event: None,
            application,
            interactions: Default::default(),
            text_renderer: TextRenderer::new(1024, 1024),
            timer: Instant::now(),
            time: 0.0,
            assets: Default::default(),
            coords_mapping: Default::default(),
            coords_mapping_scaling: Default::default(),
            missing_texutre: None,
            glyphs_texture: None,
            colored_shader: None,
            textured_shader: None,
            text_shader: None,
            #[cfg(debug_assertions)]
            debug_shader: None,
            #[cfg(debug_assertions)]
            show_raui_aabb_mode: 0,
            #[cfg(debug_assertions)]
            show_raui_aabb_key: VirtualKeyCode::F9,
            #[cfg(debug_assertions)]
            print_raui_tree_key: VirtualKeyCode::F10,
            #[cfg(debug_assertions)]
            print_raui_layout_key: VirtualKeyCode::F11,
            #[cfg(debug_assertions)]
            print_raui_interactions_key: VirtualKeyCode::F12,
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
        #[cfg(debug_assertions)]
        {
            self.debug_shader = Some(graphics.shader(DEBUG_VERTEX, DEBUG_FRAGMENT).unwrap());
        }
    }

    fn redraw(&mut self, graphics: &mut Graphics<Vertex>, control: &mut AppControl) {
        let elapsed = self.timer.elapsed();
        self.timer = Instant::now();
        self.time += elapsed.as_secs_f32();
        if let Some(callback) = self.on_update.as_mut() {
            callback(&mut self.application, control);
        }
        self.text_renderer.clear();
        if let Some(callback) = self.on_redraw.as_mut() {
            callback(
                elapsed.as_secs_f32(),
                graphics,
                &mut self.text_renderer,
                control,
            );
        }
        {
            self.application
                .view_models
                .get_mut(RenderWorkersViewModel::VIEW_MODEL)
                .unwrap()
                .write::<RenderWorkersViewModel>()
                .unwrap()
                .maintain(
                    graphics,
                    &mut self.assets,
                    self.colored_shader.as_ref().unwrap(),
                    self.textured_shader.as_ref().unwrap(),
                    self.text_shader.as_ref().unwrap(),
                );
        }
        self.assets.maintain();
        self.application.animations_delta_time = elapsed.as_secs_f32();
        self.coords_mapping = CoordsMapping::new_scaling(
            Rect {
                left: 0.0,
                right: graphics.state.main_camera.screen_size.x,
                top: 0.0,
                bottom: graphics.state.main_camera.screen_size.y,
            },
            self.coords_mapping_scaling,
        );
        if self.application.process() {
            self.assets.load(self.application.rendered_tree(), graphics);
            let mut layout_engine = DefaultLayoutEngine::new(AppTextMeasurementsEngine {
                assets: &self.assets,
            });
            let _ = self
                .application
                .layout(&self.coords_mapping, &mut layout_engine);
        } else {
            self.assets.load(self.application.rendered_tree(), graphics);
        }
        let _ = self.application.interact(&mut self.interactions);
        self.application.consume_signals();
        let matrix = graphics
            .state
            .main_camera
            .world_projection_matrix()
            .into_col_array();
        graphics.state.stream.batch_end();
        for shader in [
            self.colored_shader.clone(),
            self.textured_shader.clone(),
            self.text_shader.clone(),
            #[cfg(debug_assertions)]
            self.debug_shader.clone(),
        ] {
            graphics.state.stream.batch(GraphicsBatch {
                shader,
                uniforms: hash_map! {
                    u_image => GlowUniformValue::I1(0),
                    u_projection_view => GlowUniformValue::M4(matrix),
                    u_time => GlowUniformValue::F1(self.time),
                },
                ..Default::default()
            });
            graphics.state.stream.batch_end();
        }
        let mut converter = TesselateToGraphics {
            colored_shader: self.colored_shader.as_ref().unwrap(),
            textured_shader: self.textured_shader.as_ref().unwrap(),
            text_shader: self.text_shader.as_ref().unwrap(),
            #[cfg(debug_assertions)]
            debug_shader: self.debug_shader.as_ref(),
            glyphs_texture: self.glyphs_texture.as_ref().unwrap(),
            missing_texture: self.missing_texutre.as_ref().unwrap(),
            assets: &self.assets,
            clip_stack: Vec::with_capacity(64),
            viewport_height: graphics.state.main_camera.screen_size.y as _,
            projection_view_matrix: matrix,
        };
        let mut renderer = TesselateRenderer::new(
            &self.assets,
            &mut converter,
            &mut graphics.state.stream,
            &mut self.text_renderer,
            if cfg!(debug_assertions) {
                match self.show_raui_aabb_mode {
                    0 => None,
                    1 => Some(TessselateRendererDebug {
                        render_non_visual_nodes: false,
                    }),
                    2 => Some(TessselateRendererDebug {
                        render_non_visual_nodes: true,
                    }),
                    _ => unreachable!(),
                }
            } else {
                None
            },
        );
        let _ = self.application.render(&self.coords_mapping, &mut renderer);
        let [w, h, d] = self.text_renderer.atlas_size();
        self.glyphs_texture.as_mut().unwrap().upload(
            w as _,
            h as _,
            d as _,
            GlowTextureFormat::Monochromatic,
            Some(self.text_renderer.image()),
        );
    }

    fn event(&mut self, event: Event<()>, window: &mut Window) -> bool {
        self.interactions.event(&event, &self.coords_mapping);
        if let Event::WindowEvent {
            event: WindowEvent::Resized(_),
            ..
        } = &event
        {
            self.application.mark_dirty();
        }
        #[cfg(debug_assertions)]
        if let Event::WindowEvent {
            event: WindowEvent::KeyboardInput { input, .. },
            ..
        } = &event
        {
            if input.state == ElementState::Pressed {
                if let Some(key) = input.virtual_keycode {
                    if key == self.show_raui_aabb_key {
                        self.show_raui_aabb_mode = (self.show_raui_aabb_mode + 1) % 3;
                        println!(
                            "* SHOW RAUI LAYOUT AABB MODE: {:#?}",
                            self.show_raui_aabb_mode
                        );
                    } else if key == self.print_raui_tree_key {
                        println!("* RAUI TREE: {:#?}", self.application.rendered_tree());
                    } else if key == self.print_raui_layout_key {
                        println!("* RAUI LAYOUT: {:#?}", self.application.layout_data());
                    } else if key == self.print_raui_interactions_key {
                        println!("* RAUI INTERACTIONS: {:#?}", self.interactions);
                    }
                }
            }
        }
        self.on_event
            .as_mut()
            .map(|callback| {
                callback(
                    &mut self.application,
                    event,
                    window,
                    &mut self.interactions.engine,
                )
            })
            .unwrap_or(true)
    }
}
