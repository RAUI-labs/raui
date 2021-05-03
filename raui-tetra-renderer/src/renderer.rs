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
        reset_scissor, set_scissor, set_transform_matrix,
        text::Text,
        Color, DrawParams, Rectangle,
    },
    math::{Mat4, Vec2},
    Context,
};

fn intersect_rects(parent: Rectangle<i32>, child: Rectangle<i32>) -> Rectangle<i32> {
    if parent.intersects(&child) {
        let tl = Vec2::<i32>::max(child.top_left(), parent.top_left());
        let br = Vec2::<i32>::min(child.bottom_right(), parent.bottom_right());
        let w = br.x - tl.x;
        let h = br.y - tl.y;
        if w > 0 && h > 0 {
            let x = tl.x;
            let y = tl.y;
            return Rectangle::new(x, y, w, h);
        }
    }
    parent
}

pub struct TetraRenderer<'a> {
    context: &'a mut Context,
    resources: &'a mut TetraResources,
    clip_stack: Vec<Rectangle<i32>>,
}

impl<'a> TetraRenderer<'a> {
    pub fn new(context: &'a mut Context, resources: &'a mut TetraResources) -> Self {
        Self {
            context,
            resources,
            clip_stack: Vec::with_capacity(32),
        }
    }

    fn push_clip(&mut self, rect: Rectangle<i32>) {
        if let Some(last) = self.clip_stack.last().copied() {
            self.clip_stack.push(intersect_rects(last, rect));
        } else {
            self.clip_stack.push(rect);
        }
        self.apply_clip();
    }

    fn pop_clip(&mut self) {
        self.clip_stack.pop();
        self.apply_clip();
    }

    fn apply_clip(&mut self) {
        if let Some(rect) = self.clip_stack.last().copied() {
            set_scissor(self.context, rect);
        } else {
            reset_scissor(self.context);
        }
    }
}

impl<'a> Renderer<(), Error> for TetraRenderer<'a> {
    fn render(
        &mut self,
        tree: &WidgetUnit,
        mapping: &CoordsMapping,
        layout: &Layout,
    ) -> Result<(), Error> {
        self.clip_stack.clear();
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
                Batch::None | Batch::FontTriangles(_, _, _) => {}
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
                Batch::ClipPush(clip) => {
                    let matrix = Mat4::from_col_array(clip.matrix);
                    let tl = matrix.mul_point(Vec2::new(0.0, 0.0));
                    let tr = matrix.mul_point(Vec2::new(clip.box_size.0, 0.0));
                    let br = matrix.mul_point(Vec2::new(clip.box_size.0, clip.box_size.1));
                    let bl = matrix.mul_point(Vec2::new(0.0, clip.box_size.1));
                    let x = tl.x.min(tr.x).min(br.x).min(bl.x).round();
                    let y = tl.y.min(tr.y).min(br.y).min(bl.y).round();
                    let x2 = tl.x.max(tr.x).max(br.x).max(bl.x).round();
                    let y2 = tl.y.max(tr.y).max(br.y).max(bl.y).round();
                    let w = x2 - x;
                    let h = y2 - y;
                    self.push_clip(Rectangle::new(x as i32, y as i32, w as i32, h as i32));
                }
                Batch::ClipPop => self.pop_clip(),
            }
        }
        Ok(())
    }
}
