//! Layout engine

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
    fn layout(&mut self, mapping: &CoordsMapping, tree: &WidgetUnit) -> Result<Layout, E>;
}

struct LayoutSortedItems<'a>(Vec<(&'a WidgetId, &'a LayoutItem)>);

impl<'a> LayoutSortedItems<'a> {
    fn new(items: &'a HashMap<WidgetId, LayoutItem>) -> Self {
        let mut items = items.iter().collect::<Vec<_>>();
        items.sort_by(|a, b| a.0.path().cmp(b.0.path()));
        Self(items)
    }
}

impl std::fmt::Debug for LayoutSortedItems<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.0.iter().map(|&(k, v)| (k, v)))
            .finish()
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
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

impl Layout {
    pub fn find(&self, mut path: &str) -> Option<&LayoutItem> {
        loop {
            if let Some(item) =
                self.items
                    .iter()
                    .find_map(|(k, v)| if k.path() == path { Some(v) } else { None })
            {
                return Some(item);
            } else if let Some(index) = path.rfind('/') {
                path = &path[0..index];
            } else {
                break;
            }
        }
        None
    }

    pub fn find_or_ui_space(&self, path: &str) -> LayoutItem {
        match self.find(path) {
            Some(item) => item.to_owned(),
            None => LayoutItem {
                local_space: self.ui_space,
                ui_space: self.ui_space,
                parent: None,
            },
        }
    }

    pub fn virtual_to_real(&self, mapping: &CoordsMapping) -> Self {
        Self {
            ui_space: mapping.virtual_to_real_rect(self.ui_space, false),
            items: self
                .items
                .iter()
                .map(|(k, v)| (k.to_owned(), v.virtual_to_real(mapping)))
                .collect::<HashMap<_, _>>(),
        }
    }

    pub fn real_to_virtual(&self, mapping: &CoordsMapping) -> Self {
        Self {
            ui_space: mapping.real_to_virtual_rect(self.ui_space, false),
            items: self
                .items
                .iter()
                .map(|(k, v)| (k.to_owned(), v.real_to_virtual(mapping)))
                .collect::<HashMap<_, _>>(),
        }
    }

    pub fn rect_relative_to(&self, id: &WidgetId, to: &WidgetId) -> Option<Rect> {
        let a = self.items.get(id)?;
        let b = self.items.get(to)?;
        let x = a.ui_space.left - b.ui_space.left;
        let y = a.ui_space.top - b.ui_space.top;
        Some(Rect {
            left: x,
            right: x + a.ui_space.width(),
            top: y,
            bottom: y + a.ui_space.height(),
        })
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LayoutItem {
    pub local_space: Rect,
    pub ui_space: Rect,
    pub parent: Option<WidgetId>,
}

impl LayoutItem {
    pub fn virtual_to_real(&self, mapping: &CoordsMapping) -> Self {
        Self {
            local_space: mapping.virtual_to_real_rect(self.local_space, true),
            ui_space: mapping.virtual_to_real_rect(self.ui_space, false),
            parent: self.parent.to_owned(),
        }
    }

    pub fn real_to_virtual(&self, mapping: &CoordsMapping) -> Self {
        Self {
            local_space: mapping.real_to_virtual_rect(self.local_space, true),
            ui_space: mapping.real_to_virtual_rect(self.ui_space, false),
            parent: self.parent.to_owned(),
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

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub enum CoordsMappingScaling {
    #[default]
    None,
    Stretch(Vec2),
    FitHorizontal(Scalar),
    FitVertical(Scalar),
    FitMinimum(Vec2),
    FitMaximum(Vec2),
    FitToView(Vec2, bool),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoordsMapping {
    #[serde(default)]
    scale: Vec2,
    #[serde(default)]
    offset: Vec2,
    #[serde(default)]
    real_area: Rect,
    #[serde(default)]
    virtual_area: Rect,
}

impl Default for CoordsMapping {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl CoordsMapping {
    pub fn new(real_area: Rect) -> Self {
        Self::new_scaling(real_area, CoordsMappingScaling::None)
    }

    pub fn new_scaling(real_area: Rect, scaling: CoordsMappingScaling) -> Self {
        match scaling {
            CoordsMappingScaling::None => Self {
                scale: 1.0.into(),
                offset: Vec2::default(),
                real_area,
                virtual_area: Rect {
                    left: 0.0,
                    right: real_area.width(),
                    top: 0.0,
                    bottom: real_area.height(),
                },
            },
            CoordsMappingScaling::Stretch(size) => {
                let vw = size.x;
                let vh = size.y;
                let rw = real_area.width();
                let rh = real_area.height();
                let scale_x = rw / vw;
                let scale_y = rh / vh;
                let w = vw * scale_x;
                let h = vh * scale_y;
                Self {
                    scale: Vec2 {
                        x: scale_x,
                        y: scale_y,
                    },
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
                    scale: scale.into(),
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
                    scale: scale.into(),
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
            CoordsMappingScaling::FitToView(size, keep_aspect_ratio) => {
                let rw = real_area.width();
                let rh = real_area.height();
                let av = size.x / size.y;
                let ar = rw / rh;
                let (scale, vw, vh) = if keep_aspect_ratio {
                    let vw = size.x;
                    let vh = size.y;
                    let scale = if ar >= av { rh / vh } else { rw / vw };
                    (scale, vw, vh)
                } else if ar >= av {
                    (rh / size.y, size.x * rw / rh, size.y)
                } else {
                    (rw / size.x, size.x, size.y * rh / rw)
                };
                let w = vw * scale;
                let h = vh * scale;
                Self {
                    scale: scale.into(),
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
        }
    }

    #[inline]
    pub fn scale(&self) -> Vec2 {
        self.scale
    }

    #[inline]
    pub fn scalar_scale(&self, max: bool) -> Scalar {
        if max {
            self.scale.x.max(self.scale.y)
        } else {
            self.scale.x.min(self.scale.y)
        }
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
    pub fn virtual_to_real_vec2(&self, coord: Vec2, local_space: bool) -> Vec2 {
        if local_space {
            Vec2 {
                x: coord.x * self.scale.x,
                y: coord.y * self.scale.y,
            }
        } else {
            Vec2 {
                x: self.offset.x + (coord.x * self.scale.x),
                y: self.offset.y + (coord.y * self.scale.y),
            }
        }
    }

    #[inline]
    pub fn real_to_virtual_vec2(&self, coord: Vec2, local_space: bool) -> Vec2 {
        if local_space {
            Vec2 {
                x: coord.x / self.scale.x,
                y: coord.y / self.scale.y,
            }
        } else {
            Vec2 {
                x: (coord.x - self.offset.x) / self.scale.x,
                y: (coord.y - self.offset.y) / self.scale.y,
            }
        }
    }

    #[inline]
    pub fn virtual_to_real_rect(&self, area: Rect, local_space: bool) -> Rect {
        if local_space {
            Rect {
                left: area.left * self.scale.x,
                right: area.right * self.scale.x,
                top: area.top * self.scale.y,
                bottom: area.bottom * self.scale.y,
            }
        } else {
            Rect {
                left: self.offset.x + (area.left * self.scale.x),
                right: self.offset.x + (area.right * self.scale.x),
                top: self.offset.y + (area.top * self.scale.y),
                bottom: self.offset.y + (area.bottom * self.scale.y),
            }
        }
    }

    #[inline]
    pub fn real_to_virtual_rect(&self, area: Rect, local_space: bool) -> Rect {
        if local_space {
            Rect {
                left: area.left / self.scale.x,
                right: area.right / self.scale.x,
                top: area.top / self.scale.y,
                bottom: area.bottom / self.scale.y,
            }
        } else {
            Rect {
                left: (area.left - self.offset.x) / self.scale.x,
                right: (area.right - self.offset.x) / self.scale.x,
                top: (area.top - self.offset.y) / self.scale.y,
                bottom: (area.bottom - self.offset.y) / self.scale.y,
            }
        }
    }
}
