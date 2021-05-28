use crate::{
    tesselation::{
        Batch, BatchClipRect, BatchExternalText, Tesselation, TesselationVerticeInterleaved,
        TesselationVertices, TesselationVerticesFormat, TesselationVerticesSeparated,
        TesselationVerticesSeparatedSliceMut, TesselationVerticesSliceMut,
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
        utils::{lerp, Rect, Transform, Vec2},
    },
    Scalar,
};
use std::collections::{HashMap, VecDeque};

fn raui_to_vec2(v: Vec2) -> vek::Vec2<Scalar> {
    vek::Vec2::new(v.x, v.y)
}

fn vec2_to_raui(v: vek::Vec2<Scalar>) -> Vec2 {
    Vec2 { x: v.x, y: v.y }
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
                horizontal_align: text.horizontal_align,
                vertical_align: text.vertical_align,
                direction: text.direction,
                color: text.color,
                box_size: layout.local_space.size(),
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
    atlas_mapping: &'a HashMap<String, (String, Rect)>,
    image_sizes: &'a HashMap<String, Vec2>,
    transform_stack: VecDeque<vek::Mat4<Scalar>>,
}

impl<'a, TTE> TesselateRenderer<'a, TTE>
where
    TTE: TextTesselationEngine,
{
    pub fn new(
        vertices_format: TesselationVerticesFormat,
        text_tesselation_engine: TTE,
        atlas_mapping: &'a HashMap<String, (String, Rect)>,
        image_sizes: &'a HashMap<String, Vec2>,
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
        atlas_mapping: &'a HashMap<String, (String, Rect)>,
        image_sizes: &'a HashMap<String, Vec2>,
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

    fn push_transform(&mut self, transform: &Transform, rect: Rect) {
        let size = rect.size();
        let offset = vek::Vec2::new(rect.left, rect.top);
        let offset = vek::Mat4::<Scalar>::translation_2d(offset);
        let pivot = vek::Vec2::new(
            lerp(0.0, size.x, transform.pivot.x),
            lerp(0.0, size.y, transform.pivot.y),
        );
        let pivot = vek::Mat4::<Scalar>::translation_2d(pivot);
        let inv_pivot = pivot.inverted();
        let align = vek::Vec2::new(
            lerp(0.0, size.x, transform.align.x),
            lerp(0.0, size.y, transform.align.y),
        );
        let align = vek::Mat4::<Scalar>::translation_2d(align);
        let translate = vek::Mat4::<Scalar>::translation_2d(raui_to_vec2(transform.translation));
        let rotate = vek::Mat4::<Scalar>::rotation_z(transform.rotation);
        let scale = vek::Mat4::<Scalar>::scaling_3d(raui_to_vec2(transform.scale).with_z(1.0));
        let skew = vek::Mat4::<Scalar>::from(vek::Mat2::new(
            1.0,
            transform.skew.y.tan(),
            transform.skew.x.tan(),
            1.0,
        ));
        let matrix = offset * align * pivot * translate * rotate * scale * skew * inv_pivot;
        self.push_matrix(matrix);
    }

    fn push_transform_simple(&mut self, rect: Rect) {
        let offset = vek::Vec2::new(rect.left, rect.top);
        let offset = vek::Mat4::<Scalar>::translation_2d(offset);
        self.push_matrix(offset);
    }

    fn push_matrix(&mut self, matrix: vek::Mat4<Scalar>) {
        let matrix = self.top_transform() * matrix;
        self.transform_stack.push_back(matrix);
    }

    fn pop_transform(&mut self) {
        self.transform_stack.pop_back();
    }

    fn top_transform(&self) -> vek::Mat4<Scalar> {
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
        size: Vec2,
        scale: Vec2,
        data: &ImageBoxColor,
        result: &mut Tesselation,
    ) {
        let matrix = self.top_transform();
        let tl = vec2_to_raui(matrix.mul_point(vek::Vec2::new(0.0, 0.0)));
        let tr = vec2_to_raui(matrix.mul_point(vek::Vec2::new(size.x, 0.0)));
        let br = vec2_to_raui(matrix.mul_point(vek::Vec2::new(size.x, size.y)));
        let bl = vec2_to_raui(matrix.mul_point(vek::Vec2::new(0.0, size.y)));
        let c = data.color;
        let indices_start = result.indices.len();
        match &data.scaling {
            ImageBoxImageScaling::Stretch => {
                let vertices_start = match &mut result.vertices {
                    TesselationVertices::Separated(TesselationVerticesSeparated {
                        position,
                        tex_coord,
                        color,
                    }) => {
                        let vertices_start = position.len();
                        position.push(tl);
                        position.push(tr);
                        position.push(br);
                        position.push(bl);
                        tex_coord.push(Default::default());
                        tex_coord.push(Default::default());
                        tex_coord.push(Default::default());
                        tex_coord.push(Default::default());
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        vertices_start as Index
                    }
                    TesselationVertices::Interleaved(data) => {
                        let vertices_start = data.len();
                        data.push(TesselationVerticeInterleaved::new(
                            tl,
                            Default::default(),
                            c,
                        ));
                        data.push(TesselationVerticeInterleaved::new(
                            tr,
                            Default::default(),
                            c,
                        ));
                        data.push(TesselationVerticeInterleaved::new(
                            br,
                            Default::default(),
                            c,
                        ));
                        data.push(TesselationVerticeInterleaved::new(
                            bl,
                            Default::default(),
                            c,
                        ));
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
                d.left *= scale.x;
                d.right *= scale.x;
                d.top *= scale.y;
                d.bottom *= scale.y;
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
                let til = vec2_to_raui(matrix.mul_point(vek::Vec2::new(d.left, 0.0)));
                let tir = vec2_to_raui(matrix.mul_point(vek::Vec2::new(size.x - d.right, 0.0)));
                let itr = vec2_to_raui(matrix.mul_point(vek::Vec2::new(size.x, d.top)));
                let ibr = vec2_to_raui(matrix.mul_point(vek::Vec2::new(size.x, size.y - d.bottom)));
                let bir = vec2_to_raui(matrix.mul_point(vek::Vec2::new(size.x - d.right, size.y)));
                let bil = vec2_to_raui(matrix.mul_point(vek::Vec2::new(d.left, size.y)));
                let ibl = vec2_to_raui(matrix.mul_point(vek::Vec2::new(0.0, size.y - d.bottom)));
                let itl = vec2_to_raui(matrix.mul_point(vek::Vec2::new(0.0, d.top)));
                let itil = vec2_to_raui(matrix.mul_point(vek::Vec2::new(d.left, d.top)));
                let itir = vec2_to_raui(matrix.mul_point(vek::Vec2::new(size.x - d.right, d.top)));
                let ibir = vec2_to_raui(
                    matrix.mul_point(vek::Vec2::new(size.x - d.right, size.y - d.bottom)),
                );
                let ibil =
                    vec2_to_raui(matrix.mul_point(vek::Vec2::new(d.left, size.y - d.bottom)));
                let vertices_start = match &mut result.vertices {
                    TesselationVertices::Separated(TesselationVerticesSeparated {
                        position,
                        tex_coord,
                        color,
                    }) => {
                        let vertices_start = position.len();
                        position.push(tl);
                        position.push(til);
                        position.push(tir);
                        position.push(tr);
                        position.push(itl);
                        position.push(itil);
                        position.push(itir);
                        position.push(itr);
                        position.push(ibl);
                        position.push(ibil);
                        position.push(ibir);
                        position.push(ibr);
                        position.push(bl);
                        position.push(bil);
                        position.push(bir);
                        position.push(br);
                        tex_coord.push(Default::default());
                        tex_coord.push(Default::default());
                        tex_coord.push(Default::default());
                        tex_coord.push(Default::default());
                        tex_coord.push(Default::default());
                        tex_coord.push(Default::default());
                        tex_coord.push(Default::default());
                        tex_coord.push(Default::default());
                        tex_coord.push(Default::default());
                        tex_coord.push(Default::default());
                        tex_coord.push(Default::default());
                        tex_coord.push(Default::default());
                        tex_coord.push(Default::default());
                        tex_coord.push(Default::default());
                        tex_coord.push(Default::default());
                        tex_coord.push(Default::default());
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
                        data.push(TesselationVerticeInterleaved::new(
                            tl,
                            Default::default(),
                            c,
                        ));
                        data.push(TesselationVerticeInterleaved::new(
                            til,
                            Default::default(),
                            c,
                        ));
                        data.push(TesselationVerticeInterleaved::new(
                            tir,
                            Default::default(),
                            c,
                        ));
                        data.push(TesselationVerticeInterleaved::new(
                            tr,
                            Default::default(),
                            c,
                        ));
                        data.push(TesselationVerticeInterleaved::new(
                            itl,
                            Default::default(),
                            c,
                        ));
                        data.push(TesselationVerticeInterleaved::new(
                            itil,
                            Default::default(),
                            c,
                        ));
                        data.push(TesselationVerticeInterleaved::new(
                            itir,
                            Default::default(),
                            c,
                        ));
                        data.push(TesselationVerticeInterleaved::new(
                            itr,
                            Default::default(),
                            c,
                        ));
                        data.push(TesselationVerticeInterleaved::new(
                            ibl,
                            Default::default(),
                            c,
                        ));
                        data.push(TesselationVerticeInterleaved::new(
                            ibil,
                            Default::default(),
                            c,
                        ));
                        data.push(TesselationVerticeInterleaved::new(
                            ibir,
                            Default::default(),
                            c,
                        ));
                        data.push(TesselationVerticeInterleaved::new(
                            ibr,
                            Default::default(),
                            c,
                        ));
                        data.push(TesselationVerticeInterleaved::new(
                            bl,
                            Default::default(),
                            c,
                        ));
                        data.push(TesselationVerticeInterleaved::new(
                            bil,
                            Default::default(),
                            c,
                        ));
                        data.push(TesselationVerticeInterleaved::new(
                            bir,
                            Default::default(),
                            c,
                        ));
                        data.push(TesselationVerticeInterleaved::new(
                            br,
                            Default::default(),
                            c,
                        ));
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
        rect: Rect,
        scale: Vec2,
        data: &ImageBoxImage,
        result: &mut Tesselation,
    ) {
        let (id, srect) = match self.atlas_mapping.get(&data.id) {
            Some((id, rect)) => (id.to_owned(), *rect),
            None => (
                data.id.to_owned(),
                Rect {
                    left: 0.0,
                    right: 1.0,
                    top: 0.0,
                    bottom: 1.0,
                },
            ),
        };
        let matrix = self.top_transform();
        let tl = vec2_to_raui(matrix.mul_point(vek::Vec2::new(rect.left, rect.top)));
        let tr = vec2_to_raui(matrix.mul_point(vek::Vec2::new(rect.right, rect.top)));
        let br = vec2_to_raui(matrix.mul_point(vek::Vec2::new(rect.right, rect.bottom)));
        let bl = vec2_to_raui(matrix.mul_point(vek::Vec2::new(rect.left, rect.bottom)));
        let ctl = Vec2 {
            x: srect.left,
            y: srect.top,
        };
        let ctr = Vec2 {
            x: srect.right,
            y: srect.top,
        };
        let cbr = Vec2 {
            x: srect.right,
            y: srect.bottom,
        };
        let cbl = Vec2 {
            x: srect.left,
            y: srect.bottom,
        };
        let c = data.tint;
        let indices_start = result.indices.len();
        match &data.scaling {
            ImageBoxImageScaling::Stretch => {
                let vertices_start = match &mut result.vertices {
                    TesselationVertices::Separated(TesselationVerticesSeparated {
                        position,
                        tex_coord,
                        color,
                    }) => {
                        let vertices_start = position.len();
                        position.push(tl);
                        position.push(tr);
                        position.push(br);
                        position.push(bl);
                        tex_coord.push(ctl);
                        tex_coord.push(ctr);
                        tex_coord.push(cbr);
                        tex_coord.push(cbl);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        color.push(c);
                        vertices_start as Index
                    }
                    TesselationVertices::Interleaved(data) => {
                        let vertices_start = data.len();
                        data.push(TesselationVerticeInterleaved::new(tl, ctl, c));
                        data.push(TesselationVerticeInterleaved::new(tr, ctr, c));
                        data.push(TesselationVerticeInterleaved::new(br, cbr, c));
                        data.push(TesselationVerticeInterleaved::new(bl, cbl, c));
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
                            Vec2 {
                                x: 1.0 / size.x,
                                y: 1.0 / size.y,
                            },
                        )
                    })
                    .unwrap_or((Vec2 { x: 1.0, y: 1.0 }, Vec2 { x: 1.0, y: 1.0 }));
                let mut d = frame.destination;
                d.left *= scale.x;
                d.right *= scale.x;
                d.top *= scale.y;
                d.bottom *= scale.y;
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
                let til =
                    vec2_to_raui(matrix.mul_point(vek::Vec2::new(rect.left + d.left, rect.top)));
                let tir =
                    vec2_to_raui(matrix.mul_point(vek::Vec2::new(rect.right - d.right, rect.top)));
                let itr =
                    vec2_to_raui(matrix.mul_point(vek::Vec2::new(rect.right, rect.top + d.top)));
                let ibr = vec2_to_raui(
                    matrix.mul_point(vek::Vec2::new(rect.right, rect.bottom - d.bottom)),
                );
                let bir = vec2_to_raui(
                    matrix.mul_point(vek::Vec2::new(rect.right - d.right, rect.bottom)),
                );
                let bil =
                    vec2_to_raui(matrix.mul_point(vek::Vec2::new(rect.left + d.left, rect.bottom)));
                let ibl = vec2_to_raui(
                    matrix.mul_point(vek::Vec2::new(rect.left, rect.bottom - d.bottom)),
                );
                let itl =
                    vec2_to_raui(matrix.mul_point(vek::Vec2::new(rect.left, rect.top + d.top)));
                let itil = vec2_to_raui(
                    matrix.mul_point(vek::Vec2::new(rect.left + d.left, rect.top + d.top)),
                );
                let itir = vec2_to_raui(
                    matrix.mul_point(vek::Vec2::new(rect.right - d.right, rect.top + d.top)),
                );
                let ibir = vec2_to_raui(
                    matrix.mul_point(vek::Vec2::new(rect.right - d.right, rect.bottom - d.bottom)),
                );
                let ibil = vec2_to_raui(
                    matrix.mul_point(vek::Vec2::new(rect.left + d.left, rect.bottom - d.bottom)),
                );
                let ctil = Vec2 {
                    x: lerp(srect.left, srect.right, frame.source.left * inv_size.x),
                    y: srect.top,
                };
                let ctir = Vec2 {
                    x: lerp(
                        srect.left,
                        srect.right,
                        1.0 - frame.source.right * inv_size.x,
                    ),
                    y: srect.top,
                };
                let citr = Vec2 {
                    x: srect.right,
                    y: lerp(srect.top, srect.bottom, frame.source.top * inv_size.y),
                };
                let cibr = Vec2 {
                    x: srect.right,
                    y: lerp(
                        srect.top,
                        srect.bottom,
                        1.0 - frame.source.bottom * inv_size.y,
                    ),
                };
                let cbir = Vec2 {
                    x: lerp(
                        srect.left,
                        srect.right,
                        1.0 - frame.source.right * inv_size.x,
                    ),
                    y: srect.bottom,
                };
                let cbil = Vec2 {
                    x: lerp(srect.left, srect.right, frame.source.left * inv_size.x),
                    y: srect.bottom,
                };
                let cibl = Vec2 {
                    x: srect.left,
                    y: lerp(
                        srect.top,
                        srect.bottom,
                        1.0 - frame.source.bottom * inv_size.y,
                    ),
                };
                let citl = Vec2 {
                    x: srect.left,
                    y: lerp(srect.top, srect.bottom, frame.source.top * inv_size.y),
                };
                let citil = Vec2 {
                    x: lerp(srect.left, srect.right, frame.source.left * inv_size.x),
                    y: lerp(srect.top, srect.bottom, frame.source.top * inv_size.y),
                };
                let citir = Vec2 {
                    x: lerp(
                        srect.left,
                        srect.right,
                        1.0 - frame.source.right * inv_size.x,
                    ),
                    y: lerp(srect.top, srect.bottom, frame.source.top * inv_size.y),
                };
                let cibir = Vec2 {
                    x: lerp(
                        srect.left,
                        srect.right,
                        1.0 - frame.source.right * inv_size.x,
                    ),
                    y: lerp(
                        srect.top,
                        srect.bottom,
                        1.0 - frame.source.bottom * inv_size.y,
                    ),
                };
                let cibil = Vec2 {
                    x: lerp(srect.left, srect.right, frame.source.left * inv_size.x),
                    y: lerp(
                        srect.top,
                        srect.bottom,
                        1.0 - frame.source.bottom * inv_size.y,
                    ),
                };
                let vertices_start = match &mut result.vertices {
                    TesselationVertices::Separated(TesselationVerticesSeparated {
                        position,
                        tex_coord,
                        color,
                    }) => {
                        let vertices_start = position.len();
                        position.push(tl);
                        position.push(til);
                        position.push(tir);
                        position.push(tr);
                        position.push(itl);
                        position.push(itil);
                        position.push(itir);
                        position.push(itr);
                        position.push(ibl);
                        position.push(ibil);
                        position.push(ibir);
                        position.push(ibr);
                        position.push(bl);
                        position.push(bil);
                        position.push(bir);
                        position.push(br);
                        tex_coord.push(ctl);
                        tex_coord.push(ctil);
                        tex_coord.push(ctir);
                        tex_coord.push(ctr);
                        tex_coord.push(citl);
                        tex_coord.push(citil);
                        tex_coord.push(citir);
                        tex_coord.push(citr);
                        tex_coord.push(cibl);
                        tex_coord.push(cibil);
                        tex_coord.push(cibir);
                        tex_coord.push(cibr);
                        tex_coord.push(cbl);
                        tex_coord.push(cbil);
                        tex_coord.push(cbir);
                        tex_coord.push(cbr);
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
                        data.push(TesselationVerticeInterleaved::new(tl, ctl, c));
                        data.push(TesselationVerticeInterleaved::new(til, ctil, c));
                        data.push(TesselationVerticeInterleaved::new(tir, ctir, c));
                        data.push(TesselationVerticeInterleaved::new(tr, ctr, c));
                        data.push(TesselationVerticeInterleaved::new(itl, citl, c));
                        data.push(TesselationVerticeInterleaved::new(itil, citil, c));
                        data.push(TesselationVerticeInterleaved::new(itir, citir, c));
                        data.push(TesselationVerticeInterleaved::new(itr, citr, c));
                        data.push(TesselationVerticeInterleaved::new(ibl, cibl, c));
                        data.push(TesselationVerticeInterleaved::new(ibil, cibil, c));
                        data.push(TesselationVerticeInterleaved::new(ibir, cibir, c));
                        data.push(TesselationVerticeInterleaved::new(ibr, cibr, c));
                        data.push(TesselationVerticeInterleaved::new(bl, cbl, c));
                        data.push(TesselationVerticeInterleaved::new(bil, cbil, c));
                        data.push(TesselationVerticeInterleaved::new(bir, cbir, c));
                        data.push(TesselationVerticeInterleaved::new(br, cbr, c));
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
                    let (vertices, indices, mut batches) =
                        unit.items.iter().fold((0, 0, 0), |a, v| {
                            let v = self.count(&v.slot, layout);
                            (a.0 + v.0, a.1 + v.1, a.2 + v.2)
                        });
                    if unit.clipping {
                        batches += 2;
                    }
                    (vertices, indices, batches)
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
        local: bool,
    ) -> Result<(), Error> {
        match unit {
            WidgetUnit::None | WidgetUnit::PortalBox(_) => Ok(()),
            WidgetUnit::AreaBox(unit) => {
                if let Some(item) = layout.items.get(&unit.id) {
                    let local_space = mapping.virtual_to_real_rect(item.local_space, local);
                    self.push_transform_simple(local_space);
                    self.render_node(&unit.slot, mapping, layout, result, true)?;
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
                    let local_space = mapping.virtual_to_real_rect(item.local_space, local);
                    self.push_transform(&unit.transform, local_space);
                    if unit.clipping {
                        result.batches.push(Batch::ClipPush(BatchClipRect {
                            box_size: local_space.size(),
                            matrix: self.top_transform().into_col_array(),
                        }));
                    }
                    for (_, item) in items {
                        self.render_node(&item.slot, mapping, layout, result, true)?;
                    }
                    if unit.clipping {
                        result.batches.push(Batch::ClipPop);
                    }
                    self.pop_transform();
                    Ok(())
                } else {
                    Err(Error::WidgetHasNoLayout(unit.id.to_owned()))
                }
            }
            WidgetUnit::FlexBox(unit) => {
                if let Some(item) = layout.items.get(&unit.id) {
                    let local_space = mapping.virtual_to_real_rect(item.local_space, local);
                    self.push_transform(&unit.transform, local_space);
                    for item in &unit.items {
                        self.render_node(&item.slot, mapping, layout, result, true)?;
                    }
                    self.pop_transform();
                    Ok(())
                } else {
                    Err(Error::WidgetHasNoLayout(unit.id.to_owned()))
                }
            }
            WidgetUnit::GridBox(unit) => {
                if let Some(item) = layout.items.get(&unit.id) {
                    let local_space = mapping.virtual_to_real_rect(item.local_space, local);
                    self.push_transform(&unit.transform, local_space);
                    for item in &unit.items {
                        self.render_node(&item.slot, mapping, layout, result, true)?;
                    }
                    self.pop_transform();
                    Ok(())
                } else {
                    Err(Error::WidgetHasNoLayout(unit.id.to_owned()))
                }
            }
            WidgetUnit::SizeBox(unit) => {
                if let Some(item) = layout.items.get(&unit.id) {
                    let local_space = mapping.virtual_to_real_rect(item.local_space, local);
                    self.push_transform(&unit.transform, local_space);
                    self.render_node(&unit.slot, mapping, layout, result, true)?;
                    self.pop_transform();
                    Ok(())
                } else {
                    Err(Error::WidgetHasNoLayout(unit.id.to_owned()))
                }
            }
            WidgetUnit::ImageBox(unit) => match &unit.material {
                ImageBoxMaterial::Color(color) => {
                    if let Some(item) = layout.items.get(&unit.id) {
                        let local_space = mapping.virtual_to_real_rect(item.local_space, local);
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
                        let local_space = mapping.virtual_to_real_rect(item.local_space, local);
                        let rect = Rect {
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
                                .unwrap_or(Vec2 { x: 1.0, y: 1.0 });
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
                            Rect {
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
                    let local_space = mapping.virtual_to_real_rect(item.local_space, local);
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
                        TesselationVertices::Separated(TesselationVerticesSeparated {
                            position,
                            tex_coord,
                            color,
                        }) => {
                            let vertices_start = position.len();
                            position.resize(vertices_start + vertices, Default::default());
                            tex_coord.resize(vertices_start + vertices, Default::default());
                            color.resize(vertices_start + vertices, Default::default());
                            self.text_tesselation_engine.tesselate(
                                unit,
                                item,
                                matrix,
                                TesselationVerticesSliceMut::Separated(
                                    TesselationVerticesSeparatedSliceMut {
                                        position: &mut position
                                            [vertices_start..(vertices_start + vertices)],
                                        tex_coord: &mut tex_coord
                                            [vertices_start..(vertices_start + vertices)],
                                        color: &mut color
                                            [vertices_start..(vertices_start + vertices)],
                                    },
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
                TesselationVerticesFormat::Separated => {
                    TesselationVertices::Separated(TesselationVerticesSeparated {
                        position: Vec::with_capacity(vertices),
                        tex_coord: Vec::with_capacity(vertices),
                        color: Vec::with_capacity(vertices),
                    })
                }
                TesselationVerticesFormat::Interleaved => {
                    TesselationVertices::Interleaved(Vec::with_capacity(vertices))
                }
            },
            indices: Vec::with_capacity(indices),
            batches: Vec::with_capacity(batches),
        };
        self.render_node(tree, mapping, layout, &mut result, false)?;
        Ok(result)
    }
}
