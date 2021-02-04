use crate::{
    props::{Props, PropsDef},
    widget::{
        node::WidgetNode,
        unit::WidgetUnitData,
        utils::{Color, Rect, Transform},
        WidgetId,
    },
    Scalar,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::TryFrom};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageBoxFrame {
    pub source: Rect,
    pub destination: Rect,
    pub frame_only: bool,
}

impl From<Scalar> for ImageBoxFrame {
    fn from(v: Scalar) -> Self {
        Self {
            source: v.into(),
            destination: v.into(),
            frame_only: false,
        }
    }
}

impl From<(Scalar, bool)> for ImageBoxFrame {
    fn from((v, fo): (Scalar, bool)) -> Self {
        Self {
            source: v.into(),
            destination: v.into(),
            frame_only: fo,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageBoxImageScaling {
    Stretch,
    Frame(ImageBoxFrame),
}

impl Default for ImageBoxImageScaling {
    fn default() -> Self {
        Self::Stretch
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ImageBoxColor {
    #[serde(default)]
    pub color: Color,
    #[serde(default)]
    pub scaling: ImageBoxImageScaling,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageBoxImage {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub source_rect: Option<Rect>,
    #[serde(default)]
    pub scaling: ImageBoxImageScaling,
    #[serde(default = "ImageBoxImage::default_tint")]
    pub tint: Color,
}

impl Default for ImageBoxImage {
    fn default() -> Self {
        Self {
            id: Default::default(),
            source_rect: Default::default(),
            scaling: Default::default(),
            tint: Self::default_tint(),
        }
    }
}

impl ImageBoxImage {
    fn default_tint() -> Color {
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ImageBoxProcedural {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub parameters: HashMap<String, Scalar>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageBoxMaterial {
    Color(ImageBoxColor),
    Image(ImageBoxImage),
    Procedural(ImageBoxProcedural),
}

impl Default for ImageBoxMaterial {
    fn default() -> Self {
        Self::Color(Default::default())
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ImageBoxSizeValue {
    Fill,
    Exact(Scalar),
}

impl Default for ImageBoxSizeValue {
    fn default() -> Self {
        Self::Fill
    }
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct ImageBoxAspectRatio {
    #[serde(default)]
    pub horizontal_alignment: Scalar,
    #[serde(default)]
    pub vertical_alignment: Scalar,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ImageBox {
    #[serde(default)]
    pub id: WidgetId,
    #[serde(default)]
    pub width: ImageBoxSizeValue,
    #[serde(default)]
    pub height: ImageBoxSizeValue,
    #[serde(default)]
    pub content_keep_aspect_ratio: Option<ImageBoxAspectRatio>,
    #[serde(default)]
    pub material: ImageBoxMaterial,
    #[serde(default)]
    pub transform: Transform,
}

impl WidgetUnitData for ImageBox {
    fn id(&self) -> &WidgetId {
        &self.id
    }
}

impl TryFrom<ImageBoxNode> for ImageBox {
    type Error = ();

    fn try_from(node: ImageBoxNode) -> Result<Self, Self::Error> {
        let ImageBoxNode {
            id,
            width,
            height,
            content_keep_aspect_ratio,
            material,
            transform,
            ..
        } = node;
        Ok(Self {
            id,
            width,
            height,
            content_keep_aspect_ratio,
            material,
            transform,
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct ImageBoxNode {
    pub id: WidgetId,
    pub props: Props,
    pub width: ImageBoxSizeValue,
    pub height: ImageBoxSizeValue,
    pub content_keep_aspect_ratio: Option<ImageBoxAspectRatio>,
    pub material: ImageBoxMaterial,
    pub transform: Transform,
}

impl ImageBoxNode {
    pub fn remap_props<F>(&mut self, mut f: F)
    where
        F: FnMut(Props) -> Props,
    {
        let props = std::mem::take(&mut self.props);
        self.props = (f)(props);
    }
}

impl Into<WidgetNode> for ImageBoxNode {
    fn into(self) -> WidgetNode {
        WidgetNode::Unit(self.into())
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ImageBoxNodeDef {
    #[serde(default)]
    pub id: WidgetId,
    #[serde(default)]
    pub props: PropsDef,
    #[serde(default)]
    pub width: ImageBoxSizeValue,
    #[serde(default)]
    pub height: ImageBoxSizeValue,
    #[serde(default)]
    pub content_keep_aspect_ratio: Option<ImageBoxAspectRatio>,
    #[serde(default)]
    pub material: ImageBoxMaterial,
    #[serde(default)]
    pub transform: Transform,
}
