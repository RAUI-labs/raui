use crate::Index;
use raui_core::{
    widget::{
        unit::text::{TextBoxDirection, TextBoxHorizontalAlign, TextBoxVerticalAlign},
        utils::{Color, Vec2},
        WidgetId,
    },
    Scalar,
};
use serde::{Deserialize, Serialize};
use std::ops::Range;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BatchExternalText {
    pub text: String,
    pub font: String,
    pub size: Scalar,
    pub color: Color,
    pub box_size: Vec2,
    pub horizontal_align: TextBoxHorizontalAlign,
    pub vertical_align: TextBoxVerticalAlign,
    pub direction: TextBoxDirection,
    pub matrix: [Scalar; 16],
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BatchClipRect {
    pub box_size: Vec2,
    pub matrix: [Scalar; 16],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Batch {
    None,
    ColoredTriangles(Range<usize>),
    ImageTriangles(String, Range<usize>),
    FontTriangles(String, Scalar, Range<usize>),
    ExternalText(WidgetId, BatchExternalText),
    ClipPush(BatchClipRect),
    ClipPop,
}

impl Default for Batch {
    fn default() -> Self {
        Self::None
    }
}

impl Batch {
    fn is_continuous(&self, other: &Self) -> bool {
        #[allow(clippy::suspicious_operation_groupings)]
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::ColoredTriangles(ra), Self::ColoredTriangles(rb)) => ra.end == rb.start,
            (Self::ImageTriangles(na, ra), Self::ImageTriangles(nb, rb)) => {
                na == nb && ra.end == rb.start
            }
            (Self::FontTriangles(na, sa, ra), Self::FontTriangles(nb, sb, rb)) => {
                na == nb && (sa - sb).abs() < 1.0e-6 && ra.end == rb.start
            }
            (Self::ClipPush(_), Self::ClipPush(_)) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TesselationVerticesSeparated {
    pub position: Vec<Vec2>,
    pub tex_coord: Vec<Vec2>,
    pub color: Vec<Color>,
}

#[derive(Debug)]
pub struct TesselationVerticesSeparatedSlice<'a> {
    pub position: &'a [Vec2],
    pub tex_coord: &'a [Vec2],
    pub color: &'a [Color],
}

#[derive(Debug)]
pub struct TesselationVerticesSeparatedSliceMut<'a> {
    pub position: &'a mut [Vec2],
    pub tex_coord: &'a mut [Vec2],
    pub color: &'a mut [Color],
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TesselationVerticeInterleaved {
    pub position: Vec2,
    pub tex_coord: Vec2,
    pub color: Color,
}

impl TesselationVerticeInterleaved {
    pub fn new(position: Vec2, tex_coord: Vec2, color: Color) -> Self {
        TesselationVerticeInterleaved {
            position,
            tex_coord,
            color,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TesselationVertices {
    Separated(TesselationVerticesSeparated),
    Interleaved(Vec<TesselationVerticeInterleaved>),
}

impl TesselationVertices {
    pub fn as_separated(&self) -> Option<TesselationVerticesSeparatedSlice> {
        match &self {
            Self::Separated(TesselationVerticesSeparated {
                position,
                tex_coord,
                color,
            }) => Some(TesselationVerticesSeparatedSlice {
                position,
                tex_coord,
                color,
            }),
            _ => None,
        }
    }

    pub fn as_interleaved(&self) -> Option<&[TesselationVerticeInterleaved]> {
        match &self {
            Self::Interleaved(data) => Some(data),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum TesselationVerticesSliceMut<'a> {
    Separated(TesselationVerticesSeparatedSliceMut<'a>),
    Interleaved(&'a mut [TesselationVerticeInterleaved]),
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum TesselationVerticesFormat {
    Separated,
    Interleaved,
}

impl Default for TesselationVerticesFormat {
    fn default() -> Self {
        Self::Interleaved
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tesselation {
    pub vertices: TesselationVertices,
    pub indices: Vec<Index>,
    pub batches: Vec<Batch>,
}

impl Tesselation {
    pub fn optimize_batches(&mut self) {
        if self.batches.is_empty() {
            return;
        }
        let mut size = 1;
        for window in self.batches.windows(2) {
            if !window[0].is_continuous(&window[1]) {
                size += 1;
            }
        }
        let batches = std::mem::replace(&mut self.batches, Vec::with_capacity(size));
        for batch in batches.into_iter() {
            if self.batches.is_empty() || !self.batches.last().unwrap().is_continuous(&batch) {
                self.batches.push(batch);
            } else {
                match (self.batches.last_mut().unwrap(), batch) {
                    (Batch::ColoredTriangles(ra), Batch::ColoredTriangles(rb))
                    | (Batch::ImageTriangles(_, ra), Batch::ImageTriangles(_, rb))
                    | (Batch::FontTriangles(_, _, ra), Batch::FontTriangles(_, _, rb)) => {
                        ra.end = rb.end
                    }
                    (a @ Batch::ClipPush(_), b @ Batch::ClipPush(_)) => {
                        *a = b;
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn optimized_batches(mut self) -> Self {
        self.optimize_batches();
        self
    }
}
