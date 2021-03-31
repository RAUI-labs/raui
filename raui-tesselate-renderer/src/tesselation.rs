use crate::Index;
use raui_core::{widget::WidgetId, Scalar};
use serde::{Deserialize, Serialize};
use std::ops::Range;
use vek::Vec2;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct Position(pub Scalar, pub Scalar);

impl From<Vec2<Scalar>> for Position {
    fn from(v: Vec2<Scalar>) -> Self {
        Self(v.x, v.y)
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct TexCoord(pub Scalar, pub Scalar);

impl From<Vec2<Scalar>> for TexCoord {
    fn from(v: Vec2<Scalar>) -> Self {
        Self(v.x, v.y)
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct Color(pub Scalar, pub Scalar, pub Scalar, pub Scalar);

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BatchExternalText {
    pub text: String,
    pub font: String,
    pub size: Scalar,
    pub color: Color,
    pub box_size: (Scalar, Scalar),
    pub matrix: [Scalar; 16],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Batch {
    None,
    ColoredTriangles(Range<usize>),
    ImageTriangles(String, Range<usize>),
    FontTriangles(String, Scalar, Range<usize>),
    ExternalText(WidgetId, BatchExternalText),
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
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TesselationVertices {
    Separated(Vec<Position>, Vec<TexCoord>, Vec<Color>),
    Interleaved(Vec<(Position, TexCoord, Color)>),
}

impl TesselationVertices {
    pub fn as_separated(&self) -> Option<(&[Position], &[TexCoord], &[Color])> {
        match &self {
            Self::Separated(p, t, c) => Some((p, t, c)),
            _ => None,
        }
    }

    pub fn as_interleaved(&self) -> Option<&[(Position, TexCoord, Color)]> {
        match &self {
            Self::Interleaved(d) => Some(d),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum TesselationVerticesSliceMut<'a> {
    Separated(&'a mut [Position], &'a mut [TexCoord], &'a mut [Color]),
    Interleaved(&'a mut [(Position, TexCoord, Color)]),
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
