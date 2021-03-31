use crate::{resources::TetraResources, Error};
use raui_core::{
    layout::{CoordsMapping, Layout},
    renderer::Renderer,
    widget::{unit::WidgetUnit, utils::Vec2 as RauiVec2},
    Scalar,
};
use raui_tesselate_renderer::{
    renderer::TesselateRenderer,
    tesselation::{Batch, TesselationVerticesFormat},
};
use tetra::{
    graphics::{
        get_transform_matrix,
        mesh::{Mesh, VertexWinding},
        set_transform_matrix,
        text::Text,
        Color, DrawParams,
    },
    math::{Mat4, Vec2},
    Context,
};

pub struct TetraRenderer<'a> {
    context: &'a mut Context,
    resources: &'a mut TetraResources,
}

impl<'a> TetraRenderer<'a> {
    pub fn new(context: &'a mut Context, resources: &'a mut TetraResources) -> Self {
        Self { context, resources }
    }
}

impl<'a> Renderer<(), Error> for TetraRenderer<'a> {
    fn render(
        &mut self,
        tree: &WidgetUnit,
        mapping: &CoordsMapping,
        layout: &Layout,
    ) -> Result<(), Error> {
        for (k, t) in &self.resources.textures {
            if let Some(v) = self.resources.image_sizes.get_mut(k) {
                v.x = t.width() as Scalar;
                v.y = t.height() as Scalar;
            } else {
                self.resources.image_sizes.insert(
                    k.to_owned(),
                    RauiVec2 {
                        x: t.width() as Scalar,
                        y: t.height() as Scalar,
                    },
                );
            }
        }
        let to_remove = self
            .resources
            .image_sizes
            .keys()
            .filter(|k| !self.resources.textures.contains_key(k.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        for key in to_remove {
            self.resources.image_sizes.remove(key.as_str());
        }
        let tesselation = TesselateRenderer::with_capacity(
            TesselationVerticesFormat::Interleaved,
            (),
            &self.resources.atlas_mapping,
            &self.resources.image_sizes,
            64,
        )
        .render(tree, mapping, layout)?
        .optimized_batches();
        // TODO: optimize rendering with `mesh_data.swap()` to avoid sync point for writes and reads.
        let mesh_data = self.resources.access_mesh_data(self.context)?;
        mesh_data.write(self.context, &tesselation)?;
        let (vertices, indices) = mesh_data.read();
        let mut mesh = Mesh::indexed(vertices, indices);
        mesh.set_front_face_winding(VertexWinding::Clockwise);
        for batch in tesselation.batches {
            match batch {
                Batch::ColoredTriangles(range) => {
                    mesh.reset_texture();
                    mesh.set_draw_range(range.start, range.end - range.start);
                    mesh.draw(self.context, DrawParams::new());
                }
                Batch::ImageTriangles(id, range) => {
                    if let Some(texture) = self.resources.textures.get(&id).cloned() {
                        mesh.set_texture(texture);
                        mesh.set_draw_range(range.start, range.end - range.start);
                        mesh.draw(self.context, DrawParams::new());
                    } else {
                        return Err(Error::ImageResourceNotFound(id));
                    }
                }
                Batch::ExternalText(wid, text) => {
                    let id = format!("{}:{}", text.font, text.size as usize);
                    if let Some((font_scale, font)) = self.resources.fonts.get(&id).cloned() {
                        let old_matrix = get_transform_matrix(self.context);
                        let new_matrix = Mat4::from_col_array(text.matrix);
                        set_transform_matrix(self.context, new_matrix);
                        let scale = mapping.scale();
                        let box_size = (text.box_size.0 * font_scale, text.box_size.1 * font_scale);
                        let mut renderable = match self.resources.texts.remove(&wid) {
                            Some(mut renderable) => {
                                renderable.set_content(text.text.as_str());
                                renderable.set_font(font);
                                renderable.set_max_width(Some(box_size.0));
                                renderable
                            }
                            None => Text::wrapped(text.text.as_str(), font, box_size.0),
                        };
                        let height = renderable
                            .get_bounds(self.context)
                            .map(|rect| rect.height)
                            .unwrap_or(box_size.1);
                        let scale = if height > 0.0 {
                            let f = (box_size.1 / height).min(1.0);
                            Vec2::new(scale / font_scale, scale * f / font_scale)
                        } else {
                            Vec2::default()
                        };
                        let params = DrawParams::new().scale(scale).color(Color::rgba(
                            text.color.0,
                            text.color.1,
                            text.color.2,
                            text.color.3,
                        ));
                        renderable.draw(self.context, params);
                        self.resources.texts.insert(wid, renderable);
                        set_transform_matrix(self.context, old_matrix);
                    } else {
                        return Err(Error::FontResourceNotFound(id));
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
