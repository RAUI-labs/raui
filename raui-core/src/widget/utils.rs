use crate::Scalar;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: Scalar,
    pub y: Scalar,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Rect {
    pub left: Scalar,
    pub right: Scalar,
    pub top: Scalar,
    pub bottom: Scalar,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: Scalar,
    pub g: Scalar,
    pub b: Scalar,
    pub a: Scalar,
}
