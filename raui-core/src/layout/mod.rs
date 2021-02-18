pub mod default_layout_engine;

use crate::{
    widget::{
        unit::WidgetUnit,
        utils::{Rect, Vec2},
        WidgetId,
    },
    Scalar,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub trait LayoutEngine<E> {
    fn layout(&mut self, register_props: &CoordsMapping, tree: &WidgetUnit) -> Result<Layout, E>;
}

#[derive(Default, Clone)]
pub struct Layout {
    pub ui_space: Rect,
    pub items: HashMap<WidgetId, LayoutItem>,
}

impl std::fmt::Debug for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Layout")
            .field("ui_space", &self.ui_space)
            .field("items", &LayoutSortedItems::new(&self.items))
            .finish()
    }
}

struct LayoutSortedItems<'a>(Vec<(&'a WidgetId, &'a LayoutItem)>);

impl<'a> LayoutSortedItems<'a> {
    fn new(items: &'a HashMap<WidgetId, LayoutItem>) -> Self {
        let mut items = items.iter().collect::<Vec<_>>();
        items.sort_by(|a, b| a.0.path().cmp(b.0.path()));
        Self(items)
    }
}

impl<'a> std::fmt::Debug for LayoutSortedItems<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.0.iter().map(|&(k, v)| (k, v)))
            .finish()
    }
}

impl Layout {
    pub fn virtual_to_real(&self, mapping: &CoordsMapping) -> Self {
        Self {
            ui_space: mapping.virtual_to_real_rect(self.ui_space),
            items: self
                .items
                .iter()
                .map(|(k, v)| (k.to_owned(), v.virtual_to_real(mapping)))
                .collect::<HashMap<_, _>>(),
        }
    }

    pub fn real_to_virtual(&self, mapping: &CoordsMapping) -> Self {
        Self {
            ui_space: mapping.real_to_virtual_rect(self.ui_space),
            items: self
                .items
                .iter()
                .map(|(k, v)| (k.to_owned(), v.real_to_virtual(mapping)))
                .collect::<HashMap<_, _>>(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct LayoutNode {
    pub id: WidgetId,
    pub local_space: Rect,
    pub children: Vec<LayoutNode>,
}

impl LayoutNode {
    pub fn count(&self) -> usize {
        1 + self.children.iter().map(Self::count).sum::<usize>()
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct LayoutItem {
    pub local_space: Rect,
    pub ui_space: Rect,
}

impl LayoutItem {
    pub fn virtual_to_real(&self, mapping: &CoordsMapping) -> Self {
        Self {
            local_space: mapping.virtual_to_real_rect(self.local_space),
            ui_space: mapping.virtual_to_real_rect(self.ui_space),
        }
    }

    pub fn real_to_virtual(&self, mapping: &CoordsMapping) -> Self {
        Self {
            local_space: mapping.real_to_virtual_rect(self.local_space),
            ui_space: mapping.real_to_virtual_rect(self.ui_space),
        }
    }
}

impl LayoutEngine<()> for () {
    fn layout(&mut self, mapping: &CoordsMapping, _: &WidgetUnit) -> Result<Layout, ()> {
        Ok(Layout {
            ui_space: mapping.virtual_area(),
            items: Default::default(),
        })
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum CoordsMappingScaling {
    None,
    Fit(Vec2),
    FitHorizontal(Scalar),
    FitVertical(Scalar),
    FitMinimum(Vec2),
    FitMaximum(Vec2),
}

impl Default for CoordsMappingScaling {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoordsMapping {
    scale: Scalar,
    offset: Vec2,
    real_area: Rect,
    virtual_area: Rect,
}

impl Default for CoordsMapping {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl CoordsMapping {
    pub fn new(real_area: Rect) -> Self {
        Self {
            scale: 1.0,
            offset: Vec2::default(),
            real_area,
            virtual_area: Rect {
                left: 0.0,
                right: real_area.width(),
                top: 0.0,
                bottom: real_area.height(),
            },
        }
    }

    pub fn new_scaling(real_area: Rect, scaling: CoordsMappingScaling) -> Self {
        match scaling {
            CoordsMappingScaling::Fit(size) => {
                let vw = size.x;
                let vh = size.y;
                let rw = real_area.width();
                let rh = real_area.height();
                let va = vw / vh;
                let ra = rw / rh;
                let scale = if va >= ra { rw / vw } else { rh / vh };
                let w = vw * scale;
                let h = vh * scale;
                Self {
                    scale,
                    offset: Vec2 {
                        x: (rw - w) * 0.5,
                        y: (rh - h) * 0.5,
                    },
                    real_area,
                    virtual_area: Rect {
                        left: 0.0,
                        right: vw,
                        top: 0.0,
                        bottom: vh,
                    },
                }
            }
            CoordsMappingScaling::FitHorizontal(vw) => {
                let rw = real_area.width();
                let rh = real_area.height();
                let scale = rw / vw;
                let vh = rh / scale;
                Self {
                    scale,
                    offset: Vec2::default(),
                    real_area,
                    virtual_area: Rect {
                        left: 0.0,
                        right: vw,
                        top: 0.0,
                        bottom: vh,
                    },
                }
            }
            CoordsMappingScaling::FitVertical(vh) => {
                let rw = real_area.width();
                let rh = real_area.height();
                let scale = rh / vh;
                let vw = rw / scale;
                Self {
                    scale,
                    offset: Vec2::default(),
                    real_area,
                    virtual_area: Rect {
                        left: 0.0,
                        right: vw,
                        top: 0.0,
                        bottom: vh,
                    },
                }
            }
            CoordsMappingScaling::FitMinimum(size) => {
                if size.x < size.y {
                    Self::new_scaling(real_area, CoordsMappingScaling::FitHorizontal(size.x))
                } else {
                    Self::new_scaling(real_area, CoordsMappingScaling::FitVertical(size.y))
                }
            }
            CoordsMappingScaling::FitMaximum(size) => {
                if size.x > size.y {
                    Self::new_scaling(real_area, CoordsMappingScaling::FitHorizontal(size.x))
                } else {
                    Self::new_scaling(real_area, CoordsMappingScaling::FitVertical(size.y))
                }
            }
            _ => Self {
                scale: 1.0,
                offset: Vec2::default(),
                real_area,
                virtual_area: Rect {
                    left: 0.0,
                    right: real_area.width(),
                    top: 0.0,
                    bottom: real_area.height(),
                },
            },
        }
    }

    #[inline]
    pub fn scale(&self) -> Scalar {
        self.scale
    }

    #[inline]
    pub fn offset(&self) -> Vec2 {
        self.offset
    }

    #[inline]
    pub fn virtual_area(&self) -> Rect {
        self.virtual_area
    }

    #[inline]
    pub fn virtual_to_real_vec2(&self, coord: Vec2) -> Vec2 {
        Vec2 {
            x: self.offset.x + (coord.x * self.scale),
            y: self.offset.y + (coord.y * self.scale),
        }
    }

    #[inline]
    pub fn real_to_virtual_vec2(&self, coord: Vec2) -> Vec2 {
        Vec2 {
            x: (coord.x - self.offset.x) / self.scale,
            y: (coord.y - self.offset.y) / self.scale,
        }
    }

    #[inline]
    pub fn virtual_to_real_rect(&self, area: Rect) -> Rect {
        Rect {
            left: self.offset.x + (area.left * self.scale),
            right: self.offset.x + (area.right * self.scale),
            top: self.offset.y + (area.top * self.scale),
            bottom: self.offset.y + (area.bottom * self.scale),
        }
    }

    #[inline]
    pub fn real_to_virtual_rect(&self, area: Rect) -> Rect {
        Rect {
            left: (area.left - self.offset.x) / self.scale,
            right: (area.right - self.offset.x) / self.scale,
            top: (area.top - self.offset.y) / self.scale,
            bottom: (area.bottom - self.offset.y) / self.scale,
        }
    }
}
