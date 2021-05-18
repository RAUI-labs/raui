use crate::Error;
use raui_core::{
    widget::{
        utils::{Rect as RauiRect, Vec2 as RauiVec2},
        WidgetId,
    },
    Scalar,
};
use raui_tesselate_renderer::tesselation::{Tesselation, TesselationVerticeInterleaved};
use std::collections::HashMap;
use tetra::{
    graphics::{
        mesh::{IndexBuffer, Vertex, VertexBuffer},
        text::{Font, Text},
        Color, Texture,
    },
    math::Vec2,
    Context,
};

pub struct MeshData {
    /// [(buffer, PoT size)]
    vertices: [(VertexBuffer, usize); 2],
    /// [(buffer, PoT size)]
    indices: [(IndexBuffer, usize); 2],
    swapped: bool,
}

impl MeshData {
    pub fn new(context: &mut Context) -> Result<Self, Error> {
        Ok(Self {
            vertices: [
                (
                    match VertexBuffer::new(
                        context,
                        &[Vertex::new(Vec2::default(), Vec2::default(), Color::BLACK)],
                    ) {
                        Ok(buffer) => buffer,
                        Err(_) => return Err(Error::CannotCreateVertexBuffer),
                    },
                    0,
                ),
                (
                    match VertexBuffer::new(
                        context,
                        &[Vertex::new(Vec2::default(), Vec2::default(), Color::BLACK)],
                    ) {
                        Ok(buffer) => buffer,
                        Err(_) => return Err(Error::CannotCreateVertexBuffer),
                    },
                    0,
                ),
            ],
            indices: [
                (
                    match IndexBuffer::new(context, &[0]) {
                        Ok(buffer) => buffer,
                        Err(_) => return Err(Error::CannotCreateIndexBuffer),
                    },
                    0,
                ),
                (
                    match IndexBuffer::new(context, &[0]) {
                        Ok(buffer) => buffer,
                        Err(_) => return Err(Error::CannotCreateIndexBuffer),
                    },
                    0,
                ),
            ],
            swapped: false,
        })
    }

    pub fn swap(&mut self) {
        self.swapped = !self.swapped;
    }

    pub fn read(&self) -> (VertexBuffer, IndexBuffer) {
        let index = self.current_index();
        (
            self.vertices[index].0.clone(),
            self.indices[index].0.clone(),
        )
    }

    pub fn write(&mut self, context: &mut Context, tesselation: &Tesselation) -> Result<(), Error> {
        let index = self.current_index();
        {
            let vertices = match tesselation.vertices.as_interleaved() {
                Some(vertices) => vertices
                    .iter()
                    .map(
                        |TesselationVerticeInterleaved {
                             position,
                             tex_coord,
                             color,
                         }| {
                            Vertex::new(
                                Vec2::new(position.x, position.y),
                                Vec2::new(tex_coord.x, tex_coord.y),
                                Color::rgba(color.r, color.g, color.b, color.a),
                            )
                        },
                    )
                    .collect::<Vec<_>>(),
                None => return Err(Error::CannotUnpackVertices),
            };
            let len = vertices.len();
            let size_old = self.vertices[index].1;
            let size_new = len.checked_next_power_of_two().unwrap_or(1);
            if size_old != size_new {
                let mut new_vertices = Vec::with_capacity(size_new);
                new_vertices.extend(vertices);
                if len < size_new {
                    new_vertices.resize(
                        size_new,
                        Vertex::new(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0), Color::BLACK),
                    );
                }
                self.vertices[index] = (
                    match VertexBuffer::new(context, &new_vertices) {
                        Ok(buffer) => buffer,
                        Err(_) => return Err(Error::CannotCreateVertexBuffer),
                    },
                    size_new,
                );
            } else {
                self.vertices[index].0.set_data(context, &vertices, 0);
            }
        }
        {
            let size_old = self.indices[index].1;
            let size_new = tesselation
                .indices
                .len()
                .checked_next_power_of_two()
                .unwrap_or(1);
            if size_old != size_new {
                let mut indices = Vec::with_capacity(size_new);
                indices.extend(&tesselation.indices);
                if tesselation.indices.len() < size_new {
                    indices.resize(size_new, 0);
                }
                self.indices[index] = (
                    match IndexBuffer::new(context, &indices) {
                        Ok(buffer) => buffer,
                        Err(_) => return Err(Error::CannotCreateIndexBuffer),
                    },
                    size_new,
                );
            } else {
                self.indices[index]
                    .0
                    .set_data(context, &tesselation.indices, 0);
            }
        }
        Ok(())
    }

    fn current_index(&self) -> usize {
        if self.swapped {
            1
        } else {
            0
        }
    }
}

#[derive(Default)]
pub struct TetraResources {
    pub fonts: HashMap<String, (Scalar, Font)>,
    pub textures: HashMap<String, Texture>,
    pub atlas_mapping: HashMap<String, (String, RauiRect)>,
    pub(crate) image_sizes: HashMap<String, RauiVec2>,
    pub(crate) texts: HashMap<WidgetId, Text>,
    pub mesh_data: Option<MeshData>,
}

impl TetraResources {
    pub fn access_mesh_data(&mut self, context: &mut Context) -> Result<&mut MeshData, Error> {
        if self.mesh_data.is_none() {
            self.mesh_data = Some(MeshData::new(context)?);
        }
        Ok(self.mesh_data.as_mut().unwrap())
    }
}
