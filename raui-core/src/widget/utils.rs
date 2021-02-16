use crate::{Integer, Scalar};
use serde::{Deserialize, Serialize};
use std::{
    hash::{Hash, Hasher},
    marker::PhantomData,
};

#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    #[serde(default)]
    pub x: Scalar,
    #[serde(default)]
    pub y: Scalar,
}

impl From<Scalar> for Vec2 {
    fn from(v: Scalar) -> Self {
        Self { x: v, y: v }
    }
}

impl From<(Scalar, Scalar)> for Vec2 {
    fn from((x, y): (Scalar, Scalar)) -> Self {
        Self { x, y }
    }
}

impl From<[Scalar; 2]> for Vec2 {
    fn from([x, y]: [Scalar; 2]) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntVec2 {
    #[serde(default)]
    pub x: Integer,
    #[serde(default)]
    pub y: Integer,
}

impl From<Integer> for IntVec2 {
    fn from(v: Integer) -> Self {
        Self { x: v, y: v }
    }
}

impl From<(Integer, Integer)> for IntVec2 {
    fn from((x, y): (Integer, Integer)) -> Self {
        Self { x, y }
    }
}

impl From<[Integer; 2]> for IntVec2 {
    fn from([x, y]: [Integer; 2]) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    #[serde(default)]
    pub left: Scalar,
    #[serde(default)]
    pub right: Scalar,
    #[serde(default)]
    pub top: Scalar,
    #[serde(default)]
    pub bottom: Scalar,
}

impl From<Scalar> for Rect {
    fn from(v: Scalar) -> Self {
        Self {
            left: v,
            right: v,
            top: v,
            bottom: v,
        }
    }
}

impl From<(Scalar, Scalar)> for Rect {
    fn from((w, h): (Scalar, Scalar)) -> Self {
        Self {
            left: 0.0,
            right: w,
            top: 0.0,
            bottom: h,
        }
    }
}

impl From<[Scalar; 2]> for Rect {
    fn from([w, h]: [Scalar; 2]) -> Self {
        Self {
            left: 0.0,
            right: w,
            top: 0.0,
            bottom: h,
        }
    }
}

impl From<(Scalar, Scalar, Scalar, Scalar)> for Rect {
    fn from((left, right, top, bottom): (Scalar, Scalar, Scalar, Scalar)) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
        }
    }
}

impl From<[Scalar; 4]> for Rect {
    fn from([left, right, top, bottom]: [Scalar; 4]) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
        }
    }
}

impl Rect {
    #[inline]
    pub fn width(&self) -> Scalar {
        self.right - self.left
    }

    #[inline]
    pub fn height(&self) -> Scalar {
        self.bottom - self.top
    }

    #[inline]
    pub fn size(&self) -> Vec2 {
        Vec2 {
            x: self.width(),
            y: self.height(),
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntRect {
    #[serde(default)]
    pub left: Integer,
    #[serde(default)]
    pub right: Integer,
    #[serde(default)]
    pub top: Integer,
    #[serde(default)]
    pub bottom: Integer,
}

impl IntRect {
    #[inline]
    pub fn width(&self) -> Integer {
        self.right - self.left
    }

    #[inline]
    pub fn height(&self) -> Integer {
        self.bottom - self.top
    }

    #[inline]
    pub fn size(&self) -> IntVec2 {
        IntVec2 {
            x: self.width(),
            y: self.height(),
        }
    }
}

impl From<Integer> for IntRect {
    fn from(v: Integer) -> Self {
        Self {
            left: v,
            right: v,
            top: v,
            bottom: v,
        }
    }
}

impl From<(Integer, Integer)> for IntRect {
    fn from((w, h): (Integer, Integer)) -> Self {
        Self {
            left: 0,
            right: w,
            top: 0,
            bottom: h,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Color {
    #[serde(default)]
    pub r: Scalar,
    #[serde(default)]
    pub g: Scalar,
    #[serde(default)]
    pub b: Scalar,
    #[serde(default)]
    pub a: Scalar,
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    #[serde(default)]
    pub pivot: Vec2,
    #[serde(default)]
    pub translation: Vec2,
    #[serde(default)]
    pub rotation: Scalar,
    #[serde(default)]
    pub scale: Vec2,
    // {angle X, angle Y}
    #[serde(default)]
    pub skew: Vec2,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            pivot: Default::default(),
            translation: Default::default(),
            rotation: Default::default(),
            scale: Self::default_scale(),
            skew: Default::default(),
        }
    }
}

impl Transform {
    fn default_scale() -> Vec2 {
        Vec2 { x: 1.0, y: 1.0 }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct MemoryId<T>(usize, PhantomData<T>);

impl<T> MemoryId<T> {
    pub fn new(v: &T) -> Self {
        Self(v as *const T as usize, PhantomData)
    }
}

impl<T> From<&T> for MemoryId<T> {
    fn from(v: &T) -> Self {
        Self::new(v)
    }
}

impl<T> Hash for MemoryId<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T> PartialEq for MemoryId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for MemoryId<T> {}

#[inline]
pub fn lerp(from: Scalar, to: Scalar, factor: Scalar) -> Scalar {
    from + (to - from) * factor
}

#[inline]
pub fn lerp_clamped(from: Scalar, to: Scalar, factor: Scalar) -> Scalar {
    lerp(from, to, factor.max(0.0).min(1.0))
}
