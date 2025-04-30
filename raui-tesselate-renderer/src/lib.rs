use std::collections::HashMap;

use bytemuck::Pod;
use fontdue::{
    Font,
    layout::{
        CoordinateSystem, HorizontalAlign, Layout as TextLayout, LayoutSettings, TextStyle,
        VerticalAlign,
    },
};
use raui_core::{
    Scalar,
    layout::{CoordsMapping, Layout},
    renderer::Renderer,
    widget::{
        WidgetId,
        unit::{
            WidgetUnit,
            image::{
                ImageBoxColor, ImageBoxImage, ImageBoxImageScaling, ImageBoxMaterial,
                ImageBoxProceduralMesh,
            },
            text::{TextBoxHorizontalAlign, TextBoxVerticalAlign},
        },
        utils::{Color, Rect, Transform, Vec2, lerp},
    },
};
use spitfire_core::{Triangle, VertexStream};
use spitfire_fontdue::{TextRenderer, TextVertex};

#[derive(Debug, Clone)]
pub enum Error {
    WidgetHasNoLayout(WidgetId),
    UnsupportedImageMaterial(Box<ImageBoxMaterial>),
    FontNotFound(String),
    ImageNotFound(String),
}

pub trait TesselateVertex: Pod {
    fn apply(&mut self, position: [f32; 2], tex_coord: [f32; 3], color: [f32; 4]);
    fn transform(&mut self, matrix: vek::Mat4<f32>);
}

#[derive(Debug, Clone, PartialEq)]
pub enum TesselateBatch {
    Color,
    Image {
        id: String,
    },
    Text,
    Procedural {
        id: String,
        images: Vec<String>,
        parameters: HashMap<String, Scalar>,
    },
    ClipPush {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
    ClipPop,
    Debug,
}

pub trait TesselateResourceProvider {
    fn image_id_and_uv_and_size_by_atlas_id(&self, id: &str) -> Option<(String, Rect, Vec2)>;
    fn fonts(&self) -> &[Font];
    fn font_index_by_id(&self, id: &str) -> Option<usize>;
}

pub trait TesselateBatchConverter<B> {
    fn convert(&mut self, batch: TesselateBatch) -> Option<B>;
}

impl TesselateBatchConverter<TesselateBatch> for () {
    fn convert(&mut self, batch: TesselateBatch) -> Option<TesselateBatch> {
        Some(batch)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TessselateRendererDebug {
    pub render_non_visual_nodes: bool,
}

pub struct TesselateRenderer<'a, V, B, P, C>
where
    V: TesselateVertex + TextVertex<Color> + Default,
    B: PartialEq,
    P: TesselateResourceProvider,
    C: TesselateBatchConverter<B>,
{
    provider: &'a P,
    converter: &'a mut C,
    stream: &'a mut VertexStream<V, B>,
    text_renderer: &'a mut TextRenderer<Color>,
    transform_stack: Vec<vek::Mat4<Scalar>>,
    debug: Option<TessselateRendererDebug>,
}

impl<'a, V, B, P, C> TesselateRenderer<'a, V, B, P, C>
where
    V: TesselateVertex + TextVertex<Color> + Default,
    B: PartialEq,
    C: TesselateBatchConverter<B>,
    P: TesselateResourceProvider,
{
    pub fn new(
        provider: &'a P,
        converter: &'a mut C,
        stream: &'a mut VertexStream<V, B>,
        text_renderer: &'a mut TextRenderer<Color>,
        debug: Option<TessselateRendererDebug>,
    ) -> Self {
        Self {
            provider,
            converter,
            stream,
            text_renderer,
            transform_stack: Default::default(),
            debug,
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
        self.transform_stack.push(matrix);
    }

    fn pop_transform(&mut self) {
        self.transform_stack.pop();
    }

    fn top_transform(&self) -> vek::Mat4<Scalar> {
        self.transform_stack.last().cloned().unwrap_or_default()
    }

    fn make_vertex(position: Vec2, tex_coord: Vec2, page: Scalar, color: Color) -> V {
        let mut result = V::default();
        TesselateVertex::apply(
            &mut result,
            [position.x, position.y],
            [tex_coord.x, tex_coord.y, page],
            [color.r, color.g, color.b, color.a],
        );
        result
    }

    fn make_tiled_triangle_first(offset: usize) -> Triangle {
        Triangle { a: 0, b: 1, c: 5 }.offset(offset)
    }

    fn make_tiled_triangle_second(offset: usize) -> Triangle {
        Triangle { a: 5, b: 4, c: 0 }.offset(offset)
    }

    fn produce_color_triangles(&mut self, size: Vec2, scale: Vec2, data: &ImageBoxColor) {
        let matrix = self.top_transform();
        let tl = vec2_to_raui(matrix.mul_point(vek::Vec2::new(0.0, 0.0)));
        let tr = vec2_to_raui(matrix.mul_point(vek::Vec2::new(size.x, 0.0)));
        let br = vec2_to_raui(matrix.mul_point(vek::Vec2::new(size.x, size.y)));
        let bl = vec2_to_raui(matrix.mul_point(vek::Vec2::new(0.0, size.y)));
        let c = data.color;
        match &data.scaling {
            ImageBoxImageScaling::Stretch => {
                if let Some(batch) = self.converter.convert(TesselateBatch::Color) {
                    self.stream.batch_optimized(batch);
                    self.stream.quad([
                        Self::make_vertex(tl, Default::default(), 0.0, c),
                        Self::make_vertex(tr, Default::default(), 0.0, c),
                        Self::make_vertex(br, Default::default(), 0.0, c),
                        Self::make_vertex(bl, Default::default(), 0.0, c),
                    ]);
                }
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
                if let Some(batch) = self.converter.convert(TesselateBatch::Color) {
                    self.stream.batch_optimized(batch);
                    self.stream.extend(
                        [
                            Self::make_vertex(tl, Default::default(), 0.0, c),
                            Self::make_vertex(til, Default::default(), 0.0, c),
                            Self::make_vertex(tir, Default::default(), 0.0, c),
                            Self::make_vertex(tr, Default::default(), 0.0, c),
                            Self::make_vertex(itl, Default::default(), 0.0, c),
                            Self::make_vertex(itil, Default::default(), 0.0, c),
                            Self::make_vertex(itir, Default::default(), 0.0, c),
                            Self::make_vertex(itr, Default::default(), 0.0, c),
                            Self::make_vertex(ibl, Default::default(), 0.0, c),
                            Self::make_vertex(ibil, Default::default(), 0.0, c),
                            Self::make_vertex(ibir, Default::default(), 0.0, c),
                            Self::make_vertex(ibr, Default::default(), 0.0, c),
                            Self::make_vertex(bl, Default::default(), 0.0, c),
                            Self::make_vertex(bil, Default::default(), 0.0, c),
                            Self::make_vertex(bir, Default::default(), 0.0, c),
                            Self::make_vertex(br, Default::default(), 0.0, c),
                        ],
                        [
                            Self::make_tiled_triangle_first(0),
                            Self::make_tiled_triangle_second(0),
                            Self::make_tiled_triangle_first(1),
                            Self::make_tiled_triangle_second(1),
                            Self::make_tiled_triangle_first(2),
                            Self::make_tiled_triangle_second(2),
                            Self::make_tiled_triangle_first(4),
                            Self::make_tiled_triangle_second(4),
                            Self::make_tiled_triangle_first(6),
                            Self::make_tiled_triangle_second(6),
                            Self::make_tiled_triangle_first(8),
                            Self::make_tiled_triangle_second(8),
                            Self::make_tiled_triangle_first(9),
                            Self::make_tiled_triangle_second(9),
                            Self::make_tiled_triangle_first(10),
                            Self::make_tiled_triangle_second(10),
                        ]
                        .into_iter()
                        .chain((!frame.frame_only).then(|| Self::make_tiled_triangle_first(5)))
                        .chain((!frame.frame_only).then(|| Self::make_tiled_triangle_second(5))),
                    );
                }
            }
        }
    }

    fn produce_image_triangles(
        &mut self,
        id: String,
        uvs: Rect,
        size: Vec2,
        rect: Rect,
        scale: Vec2,
        data: &ImageBoxImage,
    ) {
        let matrix = self.top_transform();
        let tl = vec2_to_raui(matrix.mul_point(vek::Vec2::new(rect.left, rect.top)));
        let tr = vec2_to_raui(matrix.mul_point(vek::Vec2::new(rect.right, rect.top)));
        let br = vec2_to_raui(matrix.mul_point(vek::Vec2::new(rect.right, rect.bottom)));
        let bl = vec2_to_raui(matrix.mul_point(vek::Vec2::new(rect.left, rect.bottom)));
        let ctl = Vec2 {
            x: uvs.left,
            y: uvs.top,
        };
        let ctr = Vec2 {
            x: uvs.right,
            y: uvs.top,
        };
        let cbr = Vec2 {
            x: uvs.right,
            y: uvs.bottom,
        };
        let cbl = Vec2 {
            x: uvs.left,
            y: uvs.bottom,
        };
        let c = data.tint;
        match &data.scaling {
            ImageBoxImageScaling::Stretch => {
                if let Some(batch) = self.converter.convert(TesselateBatch::Image { id }) {
                    self.stream.batch_optimized(batch);
                    self.stream.quad([
                        Self::make_vertex(tl, ctl, 0.0, c),
                        Self::make_vertex(tr, ctr, 0.0, c),
                        Self::make_vertex(br, cbr, 0.0, c),
                        Self::make_vertex(bl, cbl, 0.0, c),
                    ]);
                }
            }
            ImageBoxImageScaling::Frame(frame) => {
                let inv_size = Vec2 {
                    x: 1.0 / size.x,
                    y: 1.0 / size.y,
                };
                let mut d = frame.destination;
                d.left *= scale.x;
                d.right *= scale.x;
                d.top *= scale.y;
                d.bottom *= scale.y;
                if frame.frame_keep_aspect_ratio {
                    d.left = (frame.source.left * rect.height()) / size.y;
                    d.right = (frame.source.right * rect.height()) / size.y;
                    d.top = (frame.source.top * rect.width()) / size.x;
                    d.bottom = (frame.source.bottom * rect.width()) / size.x;
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
                    x: uvs.left + frame.source.left * inv_size.x,
                    y: uvs.top,
                };
                let ctir = Vec2 {
                    x: uvs.right - frame.source.right * inv_size.x,
                    y: uvs.top,
                };
                let citr = Vec2 {
                    x: uvs.right,
                    y: uvs.top + frame.source.top * inv_size.y,
                };
                let cibr = Vec2 {
                    x: uvs.right,
                    y: uvs.bottom - frame.source.bottom * inv_size.y,
                };
                let cbir = Vec2 {
                    x: uvs.right - frame.source.right * inv_size.x,
                    y: uvs.bottom,
                };
                let cbil = Vec2 {
                    x: uvs.left + frame.source.left * inv_size.x,
                    y: uvs.bottom,
                };
                let cibl = Vec2 {
                    x: uvs.left,
                    y: uvs.bottom - frame.source.bottom * inv_size.y,
                };
                let citl = Vec2 {
                    x: uvs.left,
                    y: uvs.top + frame.source.top * inv_size.y,
                };
                let citil = Vec2 {
                    x: uvs.left + frame.source.left * inv_size.x,
                    y: uvs.top + frame.source.top * inv_size.y,
                };
                let citir = Vec2 {
                    x: uvs.right - frame.source.right * inv_size.x,
                    y: uvs.top + frame.source.top * inv_size.y,
                };
                let cibir = Vec2 {
                    x: uvs.right - frame.source.right * inv_size.x,
                    y: uvs.bottom - frame.source.bottom * inv_size.y,
                };
                let cibil = Vec2 {
                    x: uvs.left + frame.source.left * inv_size.x,
                    y: uvs.bottom - frame.source.bottom * inv_size.y,
                };
                if let Some(batch) = self.converter.convert(TesselateBatch::Image { id }) {
                    self.stream.batch_optimized(batch);
                    self.stream.extend(
                        [
                            Self::make_vertex(tl, ctl, 0.0, c),
                            Self::make_vertex(til, ctil, 0.0, c),
                            Self::make_vertex(tir, ctir, 0.0, c),
                            Self::make_vertex(tr, ctr, 0.0, c),
                            Self::make_vertex(itl, citl, 0.0, c),
                            Self::make_vertex(itil, citil, 0.0, c),
                            Self::make_vertex(itir, citir, 0.0, c),
                            Self::make_vertex(itr, citr, 0.0, c),
                            Self::make_vertex(ibl, cibl, 0.0, c),
                            Self::make_vertex(ibil, cibil, 0.0, c),
                            Self::make_vertex(ibir, cibir, 0.0, c),
                            Self::make_vertex(ibr, cibr, 0.0, c),
                            Self::make_vertex(bl, cbl, 0.0, c),
                            Self::make_vertex(bil, cbil, 0.0, c),
                            Self::make_vertex(bir, cbir, 0.0, c),
                            Self::make_vertex(br, cbr, 0.0, c),
                        ],
                        [
                            Self::make_tiled_triangle_first(0),
                            Self::make_tiled_triangle_second(0),
                            Self::make_tiled_triangle_first(1),
                            Self::make_tiled_triangle_second(1),
                            Self::make_tiled_triangle_first(2),
                            Self::make_tiled_triangle_second(2),
                            Self::make_tiled_triangle_first(4),
                            Self::make_tiled_triangle_second(4),
                            Self::make_tiled_triangle_first(6),
                            Self::make_tiled_triangle_second(6),
                            Self::make_tiled_triangle_first(8),
                            Self::make_tiled_triangle_second(8),
                            Self::make_tiled_triangle_first(9),
                            Self::make_tiled_triangle_second(9),
                            Self::make_tiled_triangle_first(10),
                            Self::make_tiled_triangle_second(10),
                        ]
                        .into_iter()
                        .chain((!frame.frame_only).then(|| Self::make_tiled_triangle_first(5)))
                        .chain((!frame.frame_only).then(|| Self::make_tiled_triangle_second(5))),
                    );
                }
            }
        }
    }

    fn produce_debug_wireframe(&mut self, size: Vec2) {
        if let Some(batch) = self.converter.convert(TesselateBatch::Debug) {
            let rect = Rect {
                left: 0.0,
                right: size.x,
                top: 0.0,
                bottom: size.y,
            };
            let matrix = self.top_transform();
            let tl = vec2_to_raui(matrix.mul_point(vek::Vec2::new(rect.left, rect.top)));
            let tr = vec2_to_raui(matrix.mul_point(vek::Vec2::new(rect.right, rect.top)));
            let br = vec2_to_raui(matrix.mul_point(vek::Vec2::new(rect.right, rect.bottom)));
            let bl = vec2_to_raui(matrix.mul_point(vek::Vec2::new(rect.left, rect.bottom)));
            self.stream.batch_optimized(batch);
            self.stream.quad([
                Self::make_vertex(tl, Default::default(), 0.0, Default::default()),
                Self::make_vertex(tr, Default::default(), 0.0, Default::default()),
                Self::make_vertex(br, Default::default(), 0.0, Default::default()),
                Self::make_vertex(bl, Default::default(), 0.0, Default::default()),
            ]);
        }
    }

    fn render_node(
        &mut self,
        unit: &WidgetUnit,
        mapping: &CoordsMapping,
        layout: &Layout,
        local: bool,
    ) -> Result<(), Error> {
        match unit {
            WidgetUnit::None | WidgetUnit::PortalBox(_) => Ok(()),
            WidgetUnit::AreaBox(unit) => {
                if let Some(item) = layout.items.get(&unit.id) {
                    let local_space = mapping.virtual_to_real_rect(item.local_space, local);
                    self.push_transform_simple(local_space);
                    self.render_node(&unit.slot, mapping, layout, true)?;
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
                    items.sort_unstable_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
                    let local_space = mapping.virtual_to_real_rect(item.local_space, local);
                    self.push_transform(&unit.transform, local_space);
                    if unit.clipping {
                        let size = local_space.size();
                        let matrix = self.top_transform();
                        let tl = matrix.mul_point(vek::Vec2::new(0.0, 0.0));
                        let tr = matrix.mul_point(vek::Vec2::new(size.x, 0.0));
                        let br = matrix.mul_point(vek::Vec2::new(size.x, size.y));
                        let bl = matrix.mul_point(vek::Vec2::new(0.0, size.y));
                        let x = tl.x.min(tr.x).min(br.x).min(bl.x).round();
                        let y = tl.y.min(tr.y).min(br.y).min(bl.y).round();
                        let x2 = tl.x.max(tr.x).max(br.x).max(bl.x).round();
                        let y2 = tl.y.max(tr.y).max(br.y).max(bl.y).round();
                        let w = x2 - x;
                        let h = y2 - y;
                        if let Some(batch) =
                            self.converter
                                .convert(TesselateBatch::ClipPush { x, y, w, h })
                        {
                            self.stream.batch(batch);
                            self.stream.batch_end();
                        }
                    }
                    for (_, item) in items {
                        self.render_node(&item.slot, mapping, layout, true)?;
                    }
                    if unit.clipping {
                        if let Some(batch) = self.converter.convert(TesselateBatch::ClipPop) {
                            self.stream.batch(batch);
                            self.stream.batch_end();
                        }
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
                        self.render_node(&item.slot, mapping, layout, true)?;
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
                        self.render_node(&item.slot, mapping, layout, true)?;
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
                    self.render_node(&unit.slot, mapping, layout, true)?;
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
                        self.produce_color_triangles(local_space.size(), mapping.scale(), color);
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
                        let (id, uvs, size) = match self
                            .provider
                            .image_id_and_uv_and_size_by_atlas_id(&image.id)
                        {
                            Some(result) => result,
                            None => return Err(Error::ImageNotFound(image.id.to_owned())),
                        };
                        let rect = if let Some(aspect) = unit.content_keep_aspect_ratio {
                            let ox = rect.left;
                            let oy = rect.top;
                            let iw = rect.width();
                            let ih = rect.height();
                            let ra = size.x / size.y;
                            let ia = iw / ih;
                            let scale = if (ra >= ia) != aspect.outside {
                                iw / size.x
                            } else {
                                ih / size.y
                            };
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
                        self.produce_image_triangles(id, uvs, size, rect, mapping.scale(), image);
                        self.pop_transform();
                        Ok(())
                    } else {
                        Err(Error::WidgetHasNoLayout(unit.id.to_owned()))
                    }
                }
                ImageBoxMaterial::Procedural(procedural) => {
                    if let Some(item) = layout.items.get(&unit.id) {
                        let local_space = mapping.virtual_to_real_rect(item.local_space, local);
                        self.push_transform(&unit.transform, local_space);
                        if let Some(batch) = self.converter.convert(TesselateBatch::Procedural {
                            id: procedural.id.to_owned(),
                            images: procedural.images.to_owned(),
                            parameters: procedural.parameters.to_owned(),
                        }) {
                            let image_mapping =
                                CoordsMapping::new_scaling(local_space, procedural.vertex_mapping);
                            self.stream.batch_optimized(batch);
                            match &procedural.mesh {
                                ImageBoxProceduralMesh::Owned(mesh) => {
                                    self.stream.extend(
                                        mesh.vertices.iter().map(|vertex| {
                                            Self::make_vertex(
                                                image_mapping
                                                    .virtual_to_real_vec2(vertex.position, false),
                                                vertex.tex_coord,
                                                vertex.page,
                                                vertex.color,
                                            )
                                        }),
                                        mesh.triangles.iter().map(|triangle| Triangle {
                                            a: triangle[0],
                                            b: triangle[1],
                                            c: triangle[2],
                                        }),
                                    );
                                }
                                ImageBoxProceduralMesh::Shared(mesh) => {
                                    self.stream.extend(
                                        mesh.vertices.iter().map(|vertex| {
                                            Self::make_vertex(
                                                image_mapping
                                                    .virtual_to_real_vec2(vertex.position, false),
                                                vertex.tex_coord,
                                                vertex.page,
                                                vertex.color,
                                            )
                                        }),
                                        mesh.triangles.iter().map(|triangle| Triangle {
                                            a: triangle[0],
                                            b: triangle[1],
                                            c: triangle[2],
                                        }),
                                    );
                                }
                                ImageBoxProceduralMesh::Generator(generator) => {
                                    let mesh = (generator)(local_space, &procedural.parameters);
                                    self.stream.extend(
                                        mesh.vertices.into_iter().map(|vertex| {
                                            Self::make_vertex(
                                                image_mapping
                                                    .virtual_to_real_vec2(vertex.position, false),
                                                vertex.tex_coord,
                                                vertex.page,
                                                vertex.color,
                                            )
                                        }),
                                        mesh.triangles.into_iter().map(|triangle| Triangle {
                                            a: triangle[0],
                                            b: triangle[1],
                                            c: triangle[2],
                                        }),
                                    );
                                }
                            }
                        }
                        self.pop_transform();
                        Ok(())
                    } else {
                        Err(Error::WidgetHasNoLayout(unit.id.to_owned()))
                    }
                }
            },
            WidgetUnit::TextBox(unit) => {
                let font_index = match self.provider.font_index_by_id(&unit.font.name) {
                    Some(index) => index,
                    None => return Err(Error::FontNotFound(unit.font.name.to_owned())),
                };
                if let Some(item) = layout.items.get(&unit.id) {
                    let local_space = mapping.virtual_to_real_rect(item.local_space, local);
                    let size = local_space.size();
                    self.push_transform(&unit.transform, local_space);
                    let matrix = self.top_transform();
                    if let Some(batch) = self.converter.convert(TesselateBatch::Text) {
                        self.stream.batch_optimized(batch);
                        self.stream.transformed(
                            |stream| {
                                let text = TextStyle::with_user_data(
                                    &unit.text,
                                    unit.font.size * mapping.scalar_scale(false),
                                    font_index,
                                    unit.color,
                                );
                                let mut layout = TextLayout::new(CoordinateSystem::PositiveYDown);
                                layout.reset(&LayoutSettings {
                                    max_width: Some(size.x),
                                    max_height: Some(size.y),
                                    horizontal_align: match unit.horizontal_align {
                                        TextBoxHorizontalAlign::Left => HorizontalAlign::Left,
                                        TextBoxHorizontalAlign::Center => HorizontalAlign::Center,
                                        TextBoxHorizontalAlign::Right => HorizontalAlign::Right,
                                    },
                                    vertical_align: match unit.vertical_align {
                                        TextBoxVerticalAlign::Top => VerticalAlign::Top,
                                        TextBoxVerticalAlign::Middle => VerticalAlign::Middle,
                                        TextBoxVerticalAlign::Bottom => VerticalAlign::Bottom,
                                    },
                                    ..Default::default()
                                });
                                layout.append(self.provider.fonts(), &text);
                                self.text_renderer.include(self.provider.fonts(), &layout);
                                self.text_renderer.render_to_stream(stream);
                            },
                            |vertex| {
                                vertex.transform(matrix);
                            },
                        );
                    }
                    self.pop_transform();
                    Ok(())
                } else {
                    Err(Error::WidgetHasNoLayout(unit.id.to_owned()))
                }
            }
        }
    }

    fn debug_render_node(
        &mut self,
        unit: &WidgetUnit,
        mapping: &CoordsMapping,
        layout: &Layout,
        local: bool,
        debug: TessselateRendererDebug,
    ) {
        match unit {
            WidgetUnit::AreaBox(unit) => {
                if let Some(item) = layout.items.get(&unit.id) {
                    let local_space = mapping.virtual_to_real_rect(item.local_space, local);
                    self.push_transform_simple(local_space);
                    self.debug_render_node(&unit.slot, mapping, layout, true, debug);
                    if debug.render_non_visual_nodes {
                        self.produce_debug_wireframe(local_space.size());
                    }
                    self.pop_transform();
                }
            }
            WidgetUnit::ContentBox(unit) => {
                if let Some(item) = layout.items.get(&unit.id) {
                    let mut items = unit
                        .items
                        .iter()
                        .map(|item| (item.layout.depth, item))
                        .collect::<Vec<_>>();
                    items.sort_unstable_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
                    let local_space = mapping.virtual_to_real_rect(item.local_space, local);
                    self.push_transform(&unit.transform, local_space);
                    for (_, item) in items {
                        self.debug_render_node(&item.slot, mapping, layout, true, debug);
                    }
                    if debug.render_non_visual_nodes {
                        self.produce_debug_wireframe(local_space.size());
                    }
                    self.pop_transform();
                }
            }
            WidgetUnit::FlexBox(unit) => {
                if let Some(item) = layout.items.get(&unit.id) {
                    let local_space = mapping.virtual_to_real_rect(item.local_space, local);
                    self.push_transform(&unit.transform, local_space);
                    for item in &unit.items {
                        self.debug_render_node(&item.slot, mapping, layout, true, debug);
                    }
                    if debug.render_non_visual_nodes {
                        self.produce_debug_wireframe(local_space.size());
                    }
                    self.pop_transform();
                }
            }
            WidgetUnit::GridBox(unit) => {
                if let Some(item) = layout.items.get(&unit.id) {
                    let local_space = mapping.virtual_to_real_rect(item.local_space, local);
                    self.push_transform(&unit.transform, local_space);
                    for item in &unit.items {
                        self.debug_render_node(&item.slot, mapping, layout, true, debug);
                    }
                    if debug.render_non_visual_nodes {
                        self.produce_debug_wireframe(local_space.size());
                    }
                    self.pop_transform();
                }
            }
            WidgetUnit::SizeBox(unit) => {
                if let Some(item) = layout.items.get(&unit.id) {
                    let local_space = mapping.virtual_to_real_rect(item.local_space, local);
                    self.push_transform(&unit.transform, local_space);
                    self.debug_render_node(&unit.slot, mapping, layout, true, debug);
                    if debug.render_non_visual_nodes {
                        self.produce_debug_wireframe(local_space.size());
                    }
                    self.pop_transform();
                }
            }
            WidgetUnit::ImageBox(unit) => {
                if let Some(item) = layout.items.get(&unit.id) {
                    let local_space = mapping.virtual_to_real_rect(item.local_space, local);
                    self.push_transform(&unit.transform, local_space);
                    self.produce_debug_wireframe(local_space.size());
                    self.pop_transform();
                }
            }
            WidgetUnit::TextBox(unit) => {
                if let Some(item) = layout.items.get(&unit.id) {
                    let local_space = mapping.virtual_to_real_rect(item.local_space, local);
                    self.push_transform(&unit.transform, local_space);
                    self.produce_debug_wireframe(local_space.size());
                    self.pop_transform();
                }
            }
            _ => {}
        }
    }
}

impl<V, B, P, C> Renderer<(), Error> for TesselateRenderer<'_, V, B, P, C>
where
    V: TesselateVertex + TextVertex<Color> + Default,
    B: PartialEq,
    C: TesselateBatchConverter<B>,
    P: TesselateResourceProvider,
{
    fn render(
        &mut self,
        tree: &WidgetUnit,
        mapping: &CoordsMapping,
        layout: &Layout,
    ) -> Result<(), Error> {
        self.transform_stack.clear();
        self.render_node(tree, mapping, layout, false)?;
        self.stream.batch_end();
        if let Some(debug) = self.debug {
            self.transform_stack.clear();
            self.debug_render_node(tree, mapping, layout, false, debug);
            self.stream.batch_end();
        }
        Ok(())
    }
}

fn raui_to_vec2(v: Vec2) -> vek::Vec2<Scalar> {
    vek::Vec2::new(v.x, v.y)
}

fn vec2_to_raui(v: vek::Vec2<Scalar>) -> Vec2 {
    Vec2 { x: v.x, y: v.y }
}
