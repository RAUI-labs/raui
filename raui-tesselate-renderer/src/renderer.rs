use crate::{
    tesselation::{
        Batch, BatchExternalText, Color, Tesselation, TesselationVertices,
        TesselationVerticesFormat, TesselationVerticesSliceMut,
    },
    Error, Index,
};
use raui_core::{
    layout::{CoordsMapping, Layout, LayoutItem},
    renderer::Renderer,
    widget::{
        unit::{
            image::{ImageBoxColor, ImageBoxImage, ImageBoxImageScaling, ImageBoxMaterial},
            text::TextBox,
            WidgetUnit,
        },
        utils::{lerp, Color as RauiColor, Rect as RauiRect, Transform, Vec2 as RauiVec2},
    },
    Scalar,
};
use std::collections::{HashMap, VecDeque};
use vek::{Mat2, Mat4, Vec2};

fn raui_to_vec2(v: RauiVec2) -> Vec2<f32> {
    Vec2::new(v.x, v.y)
}

fn raui_to_color(v: RauiColor) -> Color {
    Color(v.r, v.g, v.b, v.a)
}

pub trait TextTesselationEngine {
    fn count(&self, text: &TextBox, layout: &LayoutItem) -> (usize, usize, usize);

    fn tesselate(
        &mut self,
        text: &TextBox,
        layout: &LayoutItem,
        matrix: [Scalar; 16],
        output_vertices: TesselationVerticesSliceMut,
        output_indices: &mut [Index],
        output_batches: &mut [Batch],
    ) -> Result<(), Error>;
}

impl TextTesselationEngine for () {
    fn count(&self, _: &TextBox, _: &LayoutItem) -> (usize, usize, usize) {
        (0, 0, 1)
    }

    fn tesselate(
        &mut self,
        text: &TextBox,
        layout: &LayoutItem,
        matrix: [Scalar; 16],
        _: TesselationVerticesSliceMut,
        _: &mut [Index],
        output_batches: &mut [Batch],
    ) -> Result<(), Error> {
        output_batches[0] = Batch::ExternalText(
            text.id.to_owned(),
            BatchExternalText {
                text: text.text.to_owned(),
                font: text.font.name.to_owned(),
                size: text.font.size,
                color: raui_to_color(text.color),
                box_size: (layout.local_space.width(), layout.local_space.height()),
                matrix,
            },
        );
        Ok(())
    }
}

#[derive(Debug)]
pub struct TesselateRenderer<'a, TTE = ()>
where
    TTE: TextTesselationEngine,
{
    pub vertices_format: TesselationVerticesFormat,
    pub text_tesselation_engine: TTE,
    /// {image id: (atlas image id, inner rectangle)}
    atlas_mapping: &'a HashMap<String, (String, RauiRect)>,
    image_sizes: &'a HashMap<String, RauiVec2>,
    transform_stack: VecDeque<Mat4<Scalar>>,
}

impl<'a, TTE> TesselateRenderer<'a, TTE>
where
    TTE: TextTesselationEngine,
{
    pub fn new(
        vertices_format: TesselationVerticesFormat,
        text_tesselation_engine: TTE,
        atlas_mapping: &'a HashMap<String, (String, RauiRect)>,
        image_sizes: &'a HashMap<String, RauiVec2>,
    ) -> Self {
        Self {
            vertices_format,
            text_tesselation_engine,
            atlas_mapping,
            image_sizes,
            transform_stack: Default::default(),
        }
    }

    pub fn with_capacity(
        vertices_format: TesselationVerticesFormat,
        text_tesselation_engine: TTE,
        atlas_mapping: &'a HashMap<String, (String, RauiRect)>,
        image_sizes: &'a HashMap<String, RauiVec2>,
        transform_stack: usize,
    ) -> Self {
        Self {
            vertices_format,
            text_tesselation_engine,
            atlas_mapping,
            image_sizes,
            transform_stack: VecDeque::with_capacity(transform_stack),
        }
    }

    fn push_transform(&mut self, transform: &Transform, rect: RauiRect) {
        let size = rect.size();
        let offset = Vec2::new(rect.left, rect.top);
        let offset = Mat4::<f32>::translation_2d(offset);
        let pivot = Vec2::new(
            lerp(0.0, size.x, transform.pivot.x),
            lerp(0.0, size.y, transform.pivot.y),
        );
        let pivot = Mat4::<f32>::translation_2d(pivot);
        let inv_pivot = pivot.inverted();
        let align = Vec2::new(
            lerp(0.0, size.x, transform.align.x),
            lerp(0.0, size.y, transform.align.y),
        );
        let align = Mat4::<f32>::translation_2d(align);
        let translate = Mat4::<f32>::translation_2d(raui_to_vec2(transform.translation));
        let rotate = Mat4::<f32>::rotation_z(transform.rotation);
        let scale = Mat4::<f32>::scaling_3d(raui_to_vec2(transform.scale).with_z(1.0));
        let skew = Mat4::<f32>::from(Mat2::new(
            1.0,
            transform.skew.y.tan(),
            transform.skew.x.tan(),
            1.0,
        ));
        let matrix = offset * align * pivot * translate * rotate * scale * skew * inv_pivot;
        self.push_matrix(matrix);
    }

    fn push_transform_simple(&mut self, rect: RauiRect) {
        let offset = Vec2::new(rect.left, rect.top);
        let offset = Mat4::<f32>::translation_2d(offset);
        self.push_matrix(offset);
    }

    fn push_matrix(&mut self, matrix: Mat4<f32>) {
        let matrix = self.transform_stack.back().cloned().unwrap_or_default() * matrix;
        self.transform_stack.push_back(matrix);
    }

    fn pop_transform(&mut self) {
        self.transform_stack.pop_back();
    }

    fn top_transform(&self) -> Mat4<f32> {
        self.transform_stack.back().cloned().unwrap_or_default()
    }

    fn push_tiled_indices(indices: &mut Vec<Index>, start: Index) {
        indices.push(start);
        indices.push(start + 1);
        indices.push(start + 5);
        indices.push(start + 5);
        indices.push(start + 4);
        indices.push(start);
    }

    fn produce_color_triangles(
        &self,
        size: RauiVec2,
        scale: Scalar,
        data: &ImageBoxColor,
        result: &mut Tesselation,
    ) {
        let matrix = self.top_transform();
        let tl = matrix.mul_point(Vec2::new(0.0, 0.0));
        let tr = matrix.mul_point(Vec2::new(size.x, 0.0));
        let br = matrix.mul_point(Vec2::new(size.x, size.y));
        let bl = matrix.mul_point(Vec2::new(0.0, size.y));
        let c = raui_to_color(data.color);
        let indices_start = result.indices.len();
        match &data.scaling {
            ImageBoxImageScaling::Stretch => {
                let vertices_start = match &mut result.vertices {
                    TesselationVertices::Separated(position, texcoord, color) => {
                        let vertices_start = position.len();
                        position.push(tl.into());
                        position.push(tr.into());
                        position.push(br.into());
                        position.push(bl.into());
                        texcoord.push(Default::default());
                        texcoord.push(Default::default());
                        texcoord.push(Default::default());
                        texcoord.push(Default::default());
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        vertices_start as Index
                    }
                    TesselationVertices::Interleaved(data) => {
                        let vertices_start = data.len();
                        data.push((tl.into(), Default::default(), c));
                        data.push((tr.into(), Default::default(), c));
                        data.push((br.into(), Default::default(), c));
                        data.push((bl.into(), Default::default(), c));
                        vertices_start as Index
                    }
                };
                result.indices.push(vertices_start);
                result.indices.push(vertices_start + 1);
                result.indices.push(vertices_start + 2);
                result.indices.push(vertices_start + 2);
                result.indices.push(vertices_start + 3);
                result.indices.push(vertices_start);
                result
                    .batches
                    .push(Batch::ColoredTriangles(indices_start..(indices_start + 6)));
            }
            ImageBoxImageScaling::Frame(frame) => {
                let mut d = frame.destination;
                d.left *= scale;
                d.right *= scale;
                d.top *= scale;
                d.bottom *= scale;
                if d.left + d.right > size.x {
                    let m = d.left + d.right;
                    d.left = size.x * d.left / m;
                    d.right = size.x * d.right / m;
                }
                if d.top + d.bottom > size.y {
                    let m = d.top + d.bottom;
                    d.top = size.y * d.top / m;
                    d.bottom = size.y * d.bottom / m;
                }
                let til = matrix.mul_point(Vec2::new(d.left, 0.0));
                let tir = matrix.mul_point(Vec2::new(size.x - d.right, 0.0));
                let itr = matrix.mul_point(Vec2::new(size.x, d.top));
                let ibr = matrix.mul_point(Vec2::new(size.x, size.y - d.bottom));
                let bir = matrix.mul_point(Vec2::new(size.x - d.right, size.y));
                let bil = matrix.mul_point(Vec2::new(d.left, size.y));
                let ibl = matrix.mul_point(Vec2::new(0.0, size.y - d.bottom));
                let itl = matrix.mul_point(Vec2::new(0.0, d.top));
                let itil = matrix.mul_point(Vec2::new(d.left, d.top));
                let itir = matrix.mul_point(Vec2::new(size.x - d.right, d.top));
                let ibir = matrix.mul_point(Vec2::new(size.x - d.right, size.y - d.bottom));
                let ibil = matrix.mul_point(Vec2::new(d.left, size.y - d.bottom));
                let vertices_start = match &mut result.vertices {
                    TesselationVertices::Separated(position, texcoord, color) => {
                        let vertices_start = position.len();
                        position.push(tl.into());
                        position.push(til.into());
                        position.push(tir.into());
                        position.push(tr.into());
                        position.push(itl.into());
                        position.push(itil.into());
                        position.push(itir.into());
                        position.push(itr.into());
                        position.push(ibl.into());
                        position.push(ibil.into());
                        position.push(ibir.into());
                        position.push(ibr.into());
                        position.push(bl.into());
                        position.push(bil.into());
                        position.push(bir.into());
                        position.push(br.into());
                        texcoord.push(Default::default());
                        texcoord.push(Default::default());
                        texcoord.push(Default::default());
                        texcoord.push(Default::default());
                        texcoord.push(Default::default());
                        texcoord.push(Default::default());
                        texcoord.push(Default::default());
                        texcoord.push(Default::default());
                        texcoord.push(Default::default());
                        texcoord.push(Default::default());
                        texcoord.push(Default::default());
                        texcoord.push(Default::default());
                        texcoord.push(Default::default());
                        texcoord.push(Default::default());
                        texcoord.push(Default::default());
                        texcoord.push(Default::default());
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        vertices_start as Index
                    }
                    TesselationVertices::Interleaved(data) => {
                        let vertices_start = data.len();
                        data.push((tl.into(), Default::default(), c));
                        data.push((til.into(), Default::default(), c));
                        data.push((tir.into(), Default::default(), c));
                        data.push((tr.into(), Default::default(), c));
                        data.push((itl.into(), Default::default(), c));
                        data.push((itil.into(), Default::default(), c));
                        data.push((itir.into(), Default::default(), c));
                        data.push((itr.into(), Default::default(), c));
                        data.push((ibl.into(), Default::default(), c));
                        data.push((ibil.into(), Default::default(), c));
                        data.push((ibir.into(), Default::default(), c));
                        data.push((ibr.into(), Default::default(), c));
                        data.push((bl.into(), Default::default(), c));
                        data.push((bil.into(), Default::default(), c));
                        data.push((bir.into(), Default::default(), c));
                        data.push((br.into(), Default::default(), c));
                        vertices_start as Index
                    }
                };
                Self::push_tiled_indices(&mut result.indices, vertices_start);
                Self::push_tiled_indices(&mut result.indices, vertices_start + 1);
                Self::push_tiled_indices(&mut result.indices, vertices_start + 2);
                Self::push_tiled_indices(&mut result.indices, vertices_start + 4);
                if !frame.frame_only {
                    Self::push_tiled_indices(&mut result.indices, vertices_start + 5);
                }
                Self::push_tiled_indices(&mut result.indices, vertices_start + 6);
                Self::push_tiled_indices(&mut result.indices, vertices_start + 8);
                Self::push_tiled_indices(&mut result.indices, vertices_start + 9);
                Self::push_tiled_indices(&mut result.indices, vertices_start + 10);
                let indices_end = indices_start + if frame.frame_only { 6 * 8 } else { 6 * 9 };
                result
                    .batches
                    .push(Batch::ColoredTriangles(indices_start..indices_end));
            }
        }
    }

    fn produce_image_triangles(
        &self,
        rect: RauiRect,
        scale: Scalar,
        data: &ImageBoxImage,
        result: &mut Tesselation,
    ) {
        let (id, srect) = match self.atlas_mapping.get(&data.id) {
            Some((id, rect)) => (id.to_owned(), *rect),
            None => (
                data.id.to_owned(),
                RauiRect {
                    left: 0.0,
                    right: 1.0,
                    top: 0.0,
                    bottom: 1.0,
                },
            ),
        };
        let matrix = self.top_transform();
        let tl = matrix.mul_point(Vec2::new(rect.left, rect.top));
        let tr = matrix.mul_point(Vec2::new(rect.right, rect.top));
        let br = matrix.mul_point(Vec2::new(rect.right, rect.bottom));
        let bl = matrix.mul_point(Vec2::new(rect.left, rect.bottom));
        let ctl = Vec2::new(srect.left, srect.top);
        let ctr = Vec2::new(srect.right, srect.top);
        let cbr = Vec2::new(srect.right, srect.bottom);
        let cbl = Vec2::new(srect.left, srect.bottom);
        let c = raui_to_color(data.tint);
        let indices_start = result.indices.len();
        match &data.scaling {
            ImageBoxImageScaling::Stretch => {
                let vertices_start = match &mut result.vertices {
                    TesselationVertices::Separated(position, texcoord, color) => {
                        let vertices_start = position.len();
                        position.push(tl.into());
                        position.push(tr.into());
                        position.push(br.into());
                        position.push(bl.into());
                        texcoord.push(ctl.into());
                        texcoord.push(ctr.into());
                        texcoord.push(cbr.into());
                        texcoord.push(cbl.into());
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        vertices_start as Index
                    }
                    TesselationVertices::Interleaved(data) => {
                        let vertices_start = data.len();
                        data.push((tl.into(), ctl.into(), c));
                        data.push((tr.into(), ctr.into(), c));
                        data.push((br.into(), cbr.into(), c));
                        data.push((bl.into(), cbl.into(), c));
                        vertices_start as Index
                    }
                };
                result.indices.push(vertices_start);
                result.indices.push(vertices_start + 1);
                result.indices.push(vertices_start + 2);
                result.indices.push(vertices_start + 2);
                result.indices.push(vertices_start + 3);
                result.indices.push(vertices_start);
                result.batches.push(Batch::ImageTriangles(
                    id,
                    indices_start..(indices_start + 6),
                ));
            }
            ImageBoxImageScaling::Frame(frame) => {
                let (source_size, inv_size) = self
                    .image_sizes
                    .get(&id)
                    .map(|size| {
                        (
                            *size,
                            RauiVec2 {
                                x: 1.0 / size.x,
                                y: 1.0 / size.y,
                            },
                        )
                    })
                    .unwrap_or((RauiVec2 { x: 1.0, y: 1.0 }, RauiVec2 { x: 1.0, y: 1.0 }));
                let mut d = frame.destination;
                d.left *= scale;
                d.right *= scale;
                d.top *= scale;
                d.bottom *= scale;
                if frame.frame_keep_aspect_ratio {
                    d.left = (frame.source.left * rect.height()) / source_size.y;
                    d.right = (frame.source.right * rect.height()) / source_size.y;
                    d.top = (frame.source.top * rect.width()) / source_size.x;
                    d.bottom = (frame.source.bottom * rect.width()) / source_size.x;
                }
                if d.left + d.right > rect.width() {
                    let m = d.left + d.right;
                    d.left = rect.width() * d.left / m;
                    d.right = rect.width() * d.right / m;
                }
                if d.top + d.bottom > rect.height() {
                    let m = d.top + d.bottom;
                    d.top = rect.height() * d.top / m;
                    d.bottom = rect.height() * d.bottom / m;
                }
                let til = matrix.mul_point(Vec2::new(rect.left + d.left, rect.top));
                let tir = matrix.mul_point(Vec2::new(rect.right - d.right, rect.top));
                let itr = matrix.mul_point(Vec2::new(rect.right, rect.top + d.top));
                let ibr = matrix.mul_point(Vec2::new(rect.right, rect.bottom - d.bottom));
                let bir = matrix.mul_point(Vec2::new(rect.right - d.right, rect.bottom));
                let bil = matrix.mul_point(Vec2::new(rect.left + d.left, rect.bottom));
                let ibl = matrix.mul_point(Vec2::new(rect.left, rect.bottom - d.bottom));
                let itl = matrix.mul_point(Vec2::new(rect.left, rect.top + d.top));
                let itil = matrix.mul_point(Vec2::new(rect.left + d.left, rect.top + d.top));
                let itir = matrix.mul_point(Vec2::new(rect.right - d.right, rect.top + d.top));
                let ibir =
                    matrix.mul_point(Vec2::new(rect.right - d.right, rect.bottom - d.bottom));
                let ibil = matrix.mul_point(Vec2::new(rect.left + d.left, rect.bottom - d.bottom));
                let ctil = Vec2::new(
                    lerp(srect.left, srect.right, frame.source.left * inv_size.x),
                    srect.top,
                );
                let ctir = Vec2::new(
                    lerp(
                        srect.left,
                        srect.right,
                        1.0 - frame.source.right * inv_size.x,
                    ),
                    srect.top,
                );
                let citr = Vec2::new(
                    srect.right,
                    lerp(srect.top, srect.bottom, frame.source.top * inv_size.y),
                );
                let cibr = Vec2::new(
                    srect.right,
                    lerp(
                        srect.top,
                        srect.bottom,
                        1.0 - frame.source.bottom * inv_size.y,
                    ),
                );
                let cbir = Vec2::new(
                    lerp(
                        srect.left,
                        srect.right,
                        1.0 - frame.source.right * inv_size.x,
                    ),
                    srect.bottom,
                );
                let cbil = Vec2::new(
                    lerp(srect.left, srect.right, frame.source.left * inv_size.x),
                    srect.bottom,
                );
                let cibl = Vec2::new(
                    srect.left,
                    lerp(
                        srect.top,
                        srect.bottom,
                        1.0 - frame.source.bottom * inv_size.y,
                    ),
                );
                let citl = Vec2::new(
                    srect.left,
                    lerp(srect.top, srect.bottom, frame.source.top * inv_size.y),
                );
                let citil = Vec2::new(
                    lerp(srect.left, srect.right, frame.source.left * inv_size.x),
                    lerp(srect.top, srect.bottom, frame.source.top * inv_size.y),
                );
                let citir = Vec2::new(
                    lerp(
                        srect.left,
                        srect.right,
                        1.0 - frame.source.right * inv_size.x,
                    ),
                    lerp(srect.top, srect.bottom, frame.source.top * inv_size.y),
                );
                let cibir = Vec2::new(
                    lerp(
                        srect.left,
                        srect.right,
                        1.0 - frame.source.right * inv_size.x,
                    ),
                    lerp(
                        srect.top,
                        srect.bottom,
                        1.0 - frame.source.bottom * inv_size.y,
                    ),
                );
                let cibil = Vec2::new(
                    lerp(srect.left, srect.right, frame.source.left * inv_size.x),
                    lerp(
                        srect.top,
                        srect.bottom,
                        1.0 - frame.source.bottom * inv_size.y,
                    ),
                );
                let vertices_start = match &mut result.vertices {
                    TesselationVertices::Separated(position, texcoord, color) => {
                        let vertices_start = position.len();
                        position.push(tl.into());
                        position.push(til.into());
                        position.push(tir.into());
                        position.push(tr.into());
                        position.push(itl.into());
                        position.push(itil.into());
                        position.push(itir.into());
                        position.push(itr.into());
                        position.push(ibl.into());
                        position.push(ibil.into());
                        position.push(ibir.into());
                        position.push(ibr.into());
                        position.push(bl.into());
                        position.push(bil.into());
                        position.push(bir.into());
                        position.push(br.into());
                        texcoord.push(ctl.into());
                        texcoord.push(ctil.into());
                        texcoord.push(ctir.into());
                        texcoord.push(ctr.into());
                        texcoord.push(citl.into());
                        texcoord.push(citil.into());
                        texcoord.push(citir.into());
                        texcoord.push(citr.into());
                        texcoord.push(cibl.into());
                        texcoord.push(cibil.into());
                        texcoord.push(cibir.into());
                        texcoord.push(cibr.into());
                        texcoord.push(cbl.into());
                        texcoord.push(cbil.into());
                        texcoord.push(cbir.into());
                        texcoord.push(cbr.into());
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        vertices_start as Index
                    }
                    TesselationVertices::Interleaved(data) => {
                        let vertices_start = data.len();
                        data.push((tl.into(), ctl.into(), c));
                        data.push((til.into(), ctil.into(), c));
                        data.push((tir.into(), ctir.into(), c));
                        data.push((tr.into(), ctr.into(), c));
                        data.push((itl.into(), citl.into(), c));
                        data.push((itil.into(), citil.into(), c));
                        data.push((itir.into(), citir.into(), c));
                        data.push((itr.into(), citr.into(), c));
                        data.push((ibl.into(), cibl.into(), c));
                        data.push((ibil.into(), cibil.into(), c));
                        data.push((ibir.into(), cibir.into(), c));
                        data.push((ibr.into(), cibr.into(), c));
                        data.push((bl.into(), cbl.into(), c));
                        data.push((bil.into(), cbil.into(), c));
                        data.push((bir.into(), cbir.into(), c));
                        data.push((br.into(), cbr.into(), c));
                        vertices_start as Index
                    }
                };
                Self::push_tiled_indices(&mut result.indices, vertices_start);
                Self::push_tiled_indices(&mut result.indices, vertices_start + 1);
                Self::push_tiled_indices(&mut result.indices, vertices_start + 2);
                Self::push_tiled_indices(&mut result.indices, vertices_start + 4);
                if !frame.frame_only {
                    Self::push_tiled_indices(&mut result.indices, vertices_start + 5);
                }
                Self::push_tiled_indices(&mut result.indices, vertices_start + 6);
                Self::push_tiled_indices(&mut result.indices, vertices_start + 8);
                Self::push_tiled_indices(&mut result.indices, vertices_start + 9);
                Self::push_tiled_indices(&mut result.indices, vertices_start + 10);
                let indices_end = indices_start + if frame.frame_only { 6 * 8 } else { 6 * 9 };
                result
                    .batches
                    .push(Batch::ImageTriangles(id, indices_start..indices_end));
            }
        }
    }

    fn count(&self, unit: &WidgetUnit, layout: &Layout) -> (usize, usize, usize) {
        match unit {
            WidgetUnit::None | WidgetUnit::PortalBox(_) => (0, 0, 0),
            WidgetUnit::AreaBox(unit) => self.count(&unit.slot, layout),
            WidgetUnit::ContentBox(unit) => {
                if layout.items.contains_key(&unit.id) {
                    unit.items.iter().fold((0, 0, 0), |a, v| {
                        let v = self.count(&v.slot, layout);
                        (a.0 + v.0, a.1 + v.1, a.2 + v.2)
                    })
                } else {
                    (0, 0, 0)
                }
            }
            WidgetUnit::FlexBox(unit) => {
                if layout.items.contains_key(&unit.id) {
                    unit.items.iter().fold((0, 0, 0), |a, v| {
                        let v = self.count(&v.slot, layout);
                        (a.0 + v.0, a.1 + v.1, a.2 + v.2)
                    })
                } else {
                    (0, 0, 0)
                }
            }
            WidgetUnit::GridBox(unit) => {
                if layout.items.contains_key(&unit.id) {
                    unit.items.iter().fold((0, 0, 0), |a, v| {
                        let v = self.count(&v.slot, layout);
                        (a.0 + v.0, a.1 + v.1, a.2 + v.2)
                    })
                } else {
                    (0, 0, 0)
                }
            }
            WidgetUnit::SizeBox(unit) => self.count(&unit.slot, layout),
            WidgetUnit::ImageBox(unit) => match &unit.material {
                ImageBoxMaterial::Color(color) => {
                    if layout.items.contains_key(&unit.id) {
                        match &color.scaling {
                            ImageBoxImageScaling::Stretch => (4, 6, 1),
                            ImageBoxImageScaling::Frame(frame) => {
                                if frame.frame_only {
                                    (16, 8 * 6, 1)
                                } else {
                                    (16, 9 * 6, 1)
                                }
                            }
                        }
                    } else {
                        (0, 0, 0)
                    }
                }
                ImageBoxMaterial::Image(image) => {
                    if layout.items.contains_key(&unit.id) {
                        match &image.scaling {
                            ImageBoxImageScaling::Stretch => (4, 6, 1),
                            ImageBoxImageScaling::Frame(frame) => {
                                if frame.frame_only {
                                    (16, 8 * 6, 1)
                                } else {
                                    (16, 9 * 6, 1)
                                }
                            }
                        }
                    } else {
                        (0, 0, 0)
                    }
                }
                _ => (0, 0, 0),
            },
            WidgetUnit::TextBox(unit) => {
                if let Some(item) = layout.items.get(&unit.id) {
                    self.text_tesselation_engine.count(unit, item)
                } else {
                    (0, 0, 0)
                }
            }
        }
    }

    fn render_node(
        &mut self,
        unit: &WidgetUnit,
        mapping: &CoordsMapping,
        layout: &Layout,
        result: &mut Tesselation,
    ) -> Result<(), Error> {
        match unit {
            WidgetUnit::None | WidgetUnit::PortalBox(_) => Ok(()),
            WidgetUnit::AreaBox(unit) => {
                if let Some(item) = layout.items.get(&unit.id) {
                    let local_space = mapping.virtual_to_real_rect(item.local_space);
                    self.push_transform_simple(local_space);
                    self.render_node(&unit.slot, mapping, layout, result)?;
                    self.pop_transform();
                    Ok(())
                } else {
                    Err(Error::WidgetHasNoLayout(unit.id.to_owned()))
                }
            }
            WidgetUnit::ContentBox(unit) => {
                if let Some(item) = layout.items.get(&unit.id) {
                    let mut items = unit
                        .items
                        .iter()
                        .map(|item| (item.layout.depth, item))
                        .collect::<Vec<_>>();
                    items.sort_by(|(a, _), (b, _)| a.partial_cmp(&b).unwrap());
                    let local_space = mapping.virtual_to_real_rect(item.local_space);
                    self.push_transform(&unit.transform, local_space);
                    for (_, item) in items {
                        self.render_node(&item.slot, mapping, layout, result)?;
                    }
                    self.pop_transform();
                    Ok(())
                } else {
                    Err(Error::WidgetHasNoLayout(unit.id.to_owned()))
                }
            }
            WidgetUnit::FlexBox(unit) => {
                if let Some(item) = layout.items.get(&unit.id) {
                    let local_space = mapping.virtual_to_real_rect(item.local_space);
                    self.push_transform(&unit.transform, local_space);
                    for item in &unit.items {
                        self.render_node(&item.slot, mapping, layout, result)?;
                    }
                    self.pop_transform();
                    Ok(())
                } else {
                    Err(Error::WidgetHasNoLayout(unit.id.to_owned()))
                }
            }
            WidgetUnit::GridBox(unit) => {
                if let Some(item) = layout.items.get(&unit.id) {
                    let local_space = mapping.virtual_to_real_rect(item.local_space);
                    self.push_transform(&unit.transform, local_space);
                    for item in &unit.items {
                        self.render_node(&item.slot, mapping, layout, result)?;
                    }
                    self.pop_transform();
                    Ok(())
                } else {
                    Err(Error::WidgetHasNoLayout(unit.id.to_owned()))
                }
            }
            WidgetUnit::SizeBox(unit) => {
                if let Some(item) = layout.items.get(&unit.id) {
                    let local_space = mapping.virtual_to_real_rect(item.local_space);
                    self.push_transform(&unit.transform, local_space);
                    self.render_node(&unit.slot, mapping, layout, result)?;
                    self.pop_transform();
                    Ok(())
                } else {
                    Err(Error::WidgetHasNoLayout(unit.id.to_owned()))
                }
            }
            WidgetUnit::ImageBox(unit) => match &unit.material {
                ImageBoxMaterial::Color(color) => {
                    if let Some(item) = layout.items.get(&unit.id) {
                        let local_space = mapping.virtual_to_real_rect(item.local_space);
                        self.push_transform(&unit.transform, local_space);
                        self.produce_color_triangles(
                            local_space.size(),
                            mapping.scale(),
                            color,
                            result,
                        );
                        self.pop_transform();
                        Ok(())
                    } else {
                        Err(Error::WidgetHasNoLayout(unit.id.to_owned()))
                    }
                }
                ImageBoxMaterial::Image(image) => {
                    if let Some(item) = layout.items.get(&unit.id) {
                        let local_space = mapping.virtual_to_real_rect(item.local_space);
                        let rect = RauiRect {
                            left: 0.0,
                            right: local_space.width(),
                            top: 0.0,
                            bottom: local_space.height(),
                        };
                        let rect = if let Some(aspect) = unit.content_keep_aspect_ratio {
                            let size = self
                                .image_sizes
                                .get(&image.id)
                                .cloned()
                                .unwrap_or(RauiVec2 { x: 1.0, y: 1.0 });
                            let ox = rect.left;
                            let oy = rect.top;
                            let iw = rect.width();
                            let ih = rect.height();
                            let ra = size.x / size.y;
                            let ia = iw / ih;
                            let scale = if ra >= ia { iw / size.x } else { ih / size.y };
                            let w = size.x * scale;
                            let h = size.y * scale;
                            let ow = lerp(0.0, iw - w, aspect.horizontal_alignment);
                            let oh = lerp(0.0, ih - h, aspect.vertical_alignment);
                            RauiRect {
                                left: ox + ow,
                                right: ox + ow + w,
                                top: oy + oh,
                                bottom: oy + oh + h,
                            }
                        } else {
                            rect
                        };
                        self.push_transform(&unit.transform, local_space);
                        self.produce_image_triangles(rect, mapping.scale(), image, result);
                        self.pop_transform();
                        Ok(())
                    } else {
                        Err(Error::WidgetHasNoLayout(unit.id.to_owned()))
                    }
                }
                _ => Err(Error::UnsupportedImageMaterial(unit.material.clone())),
            },
            WidgetUnit::TextBox(unit) => {
                if let Some(item) = layout.items.get(&unit.id) {
                    let local_space = mapping.virtual_to_real_rect(item.local_space);
                    self.push_transform(&unit.transform, local_space);
                    let matrix = self.top_transform().into_col_array();
                    let (vertices, indices, batches) =
                        self.text_tesselation_engine.count(unit, item);
                    let batches_start = result.batches.len();
                    let indices_start = result.indices.len();
                    result
                        .batches
                        .resize(batches_start + batches, Default::default());
                    result.indices.resize(indices_start + indices, 0);
                    let vertices_start = match &mut result.vertices {
                        TesselationVertices::Separated(position, texcoord, color) => {
                            let vertices_start = position.len();
                            position.resize(vertices_start + vertices, Default::default());
                            texcoord.resize(vertices_start + vertices, Default::default());
                            color.resize(vertices_start + vertices, Default::default());
                            self.text_tesselation_engine.tesselate(
                                unit,
                                item,
                                matrix,
                                TesselationVerticesSliceMut::Separated(
                                    &mut position[vertices_start..(vertices_start + vertices)],
                                    &mut texcoord[vertices_start..(vertices_start + vertices)],
                                    &mut color[vertices_start..(vertices_start + vertices)],
                                ),
                                &mut result.indices[indices_start..(indices_start + indices)],
                                &mut result.batches[batches_start..(batches_start + batches)],
                            )?;
                            vertices_start as Index
                        }
                        TesselationVertices::Interleaved(data) => {
                            let vertices_start = data.len();
                            data.resize(vertices_start + vertices, Default::default());
                            self.text_tesselation_engine.tesselate(
                                unit,
                                item,
                                matrix,
                                TesselationVerticesSliceMut::Interleaved(
                                    &mut data[vertices_start..(vertices_start + vertices)],
                                ),
                                &mut result.indices[indices_start..(indices_start + indices)],
                                &mut result.batches[batches_start..(batches_start + batches)],
                            )?;
                            vertices_start as Index
                        }
                    };
                    for item in &mut result.indices[indices_start..(indices_start + indices)] {
                        *item += vertices_start;
                    }
                    for item in &mut result.batches[batches_start..(batches_start + batches)] {
                        match item {
                            Batch::ColoredTriangles(range) => {
                                range.start += indices_start;
                                range.end += indices_start;
                            }
                            Batch::ImageTriangles(_, range) => {
                                range.start += indices_start;
                                range.end += indices_start;
                            }
                            Batch::FontTriangles(_, _, range) => {
                                range.start += indices_start;
                                range.end += indices_start;
                            }
                            _ => {}
                        }
                    }
                    self.pop_transform();
                    Ok(())
                } else {
                    Err(Error::WidgetHasNoLayout(unit.id.to_owned()))
                }
            }
        }
    }
}

impl<'a, TTE> Renderer<Tesselation, Error> for TesselateRenderer<'a, TTE>
where
    TTE: TextTesselationEngine,
{
    fn render(
        &mut self,
        tree: &WidgetUnit,
        mapping: &CoordsMapping,
        layout: &Layout,
    ) -> Result<Tesselation, Error> {
        self.transform_stack.clear();
        let (vertices, indices, batches) = self.count(tree, layout);
        let mut result = Tesselation {
            vertices: match self.vertices_format {
                TesselationVerticesFormat::Separated => TesselationVertices::Separated(
                    Vec::with_capacity(vertices),
                    Vec::with_capacity(vertices),
                    Vec::with_capacity(vertices),
                ),
                TesselationVerticesFormat::Interleaved => {
                    TesselationVertices::Interleaved(Vec::with_capacity(vertices))
                }
            },
            indices: Vec::with_capacity(indices),
            batches: Vec::with_capacity(batches),
        };
        self.render_node(tree, mapping, layout, &mut result)?;
        Ok(result)
    }
}
