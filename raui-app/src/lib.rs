pub mod app;
pub(crate) mod asset_manager;
pub(crate) mod interactions;

use crate::asset_manager::AssetsManager;
use bytemuck::{Pod, Zeroable};
use raui_core::widget::utils::Color;
use raui_tesselate_renderer::{TesselateBatch, TesselateBatchConverter, TesselateVertex};
use spitfire_fontdue::TextVertex;
use spitfire_glow::{
    graphics::Texture,
    prelude::{GraphicsBatch, Shader},
    renderer::{GlowBlending, GlowTextureFiltering, GlowVertexAttrib, GlowVertexAttribs},
};
use vek::Rect;

pub use glutin::{event, window};

pub mod prelude {
    pub use crate::{
        app::*,
        app::{declarative::*, immediate::*, retained::*},
        event::*,
        window::*,
    };
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 2],
    pub uv: [f32; 3],
    pub color: [f32; 4],
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            position: Default::default(),
            uv: Default::default(),
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

impl GlowVertexAttribs for Vertex {
    const ATTRIBS: &'static [(&'static str, GlowVertexAttrib)] = &[
        (
            "a_position",
            GlowVertexAttrib::Float {
                channels: 2,
                normalized: false,
            },
        ),
        (
            "a_uv",
            GlowVertexAttrib::Float {
                channels: 3,
                normalized: false,
            },
        ),
        (
            "a_color",
            GlowVertexAttrib::Float {
                channels: 4,
                normalized: false,
            },
        ),
    ];
}

impl TesselateVertex for Vertex {
    fn apply(&mut self, position: [f32; 2], tex_coord: [f32; 3], color: [f32; 4]) {
        self.position = position;
        self.uv = tex_coord;
        self.color = color;
    }

    fn transform(&mut self, matrix: vek::Mat4<f32>) {
        let result = matrix.mul_point(vek::Vec3 {
            x: self.position[0],
            y: self.position[1],
            z: 0.0,
        });
        self.position[0] = result.x;
        self.position[1] = result.y;
    }
}

impl TextVertex<Color> for Vertex {
    fn apply(&mut self, position: [f32; 2], tex_coord: [f32; 3], color: Color) {
        self.position = position;
        self.uv = tex_coord;
        self.color = [color.r, color.g, color.b, color.a];
    }
}

pub(crate) struct TesselateToGraphics<'a> {
    colored_shader: &'a Shader,
    textured_shader: &'a Shader,
    text_shader: &'a Shader,
    glyphs_texture: &'a Texture,
    missing_texture: &'a Texture,
    assets: &'a AssetsManager,
    clip_stack: Vec<Rect<i32, i32>>,
    viewport_height: i32,
}

impl<'a> TesselateBatchConverter<GraphicsBatch> for TesselateToGraphics<'a> {
    fn convert(&mut self, batch: TesselateBatch) -> Option<GraphicsBatch> {
        match batch {
            TesselateBatch::Color => Some(GraphicsBatch {
                shader: Some(self.colored_shader.clone()),
                blending: GlowBlending::Alpha,
                scissor: self.clip_stack.last().copied(),
                ..Default::default()
            }),
            TesselateBatch::Image { id } => {
                let id = AssetsManager::parse_image_id(&id).0;
                Some(GraphicsBatch {
                    shader: Some(self.textured_shader.clone()),
                    textures: vec![(
                        self.assets
                            .textures
                            .get(id)
                            .map(|texture| texture.texture.clone())
                            .unwrap_or_else(|| self.missing_texture.clone()),
                        GlowTextureFiltering::Linear,
                    )],
                    blending: GlowBlending::Alpha,
                    scissor: self.clip_stack.last().copied(),
                    ..Default::default()
                })
            }
            TesselateBatch::Text => Some(GraphicsBatch {
                shader: Some(self.text_shader.clone()),
                textures: vec![(self.glyphs_texture.clone(), GlowTextureFiltering::Linear)],
                blending: GlowBlending::Alpha,
                scissor: self.clip_stack.last().copied(),
                ..Default::default()
            }),
            TesselateBatch::ClipPush { x, y, w, h } => {
                self.clip_stack.push(vek::Rect {
                    x: x as _,
                    y: self.viewport_height - y as i32 - h as i32,
                    w: w as _,
                    h: h as _,
                });
                None
            }
            TesselateBatch::ClipPop => {
                self.clip_stack.pop();
                None
            }
        }
    }
}
