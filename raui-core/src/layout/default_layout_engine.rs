use crate::{
    Scalar,
    layout::{CoordsMapping, Layout, LayoutEngine, LayoutItem, LayoutNode},
    widget::{
        WidgetId,
        unit::{
            WidgetUnit,
            area::AreaBox,
            content::ContentBox,
            flex::FlexBox,
            grid::GridBox,
            image::{ImageBox, ImageBoxSizeValue},
            size::{SizeBox, SizeBoxAspectRatio, SizeBoxSizeValue},
            text::{TextBox, TextBoxSizeValue},
        },
        utils::{Rect, Vec2, lerp},
    },
};
use std::collections::HashMap;

pub trait TextMeasurementEngine {
    fn measure_text(
        &self,
        size_available: Vec2,
        mapping: &CoordsMapping,
        widget: &TextBox,
    ) -> Option<Rect>;
}

impl TextMeasurementEngine for () {
    fn measure_text(&self, _: Vec2, _: &CoordsMapping, _: &TextBox) -> Option<Rect> {
        None
    }
}

pub struct DefaultLayoutEngine<TME: TextMeasurementEngine = ()> {
    text_measurement_engine: TME,
}

impl<TME: TextMeasurementEngine + Default> Default for DefaultLayoutEngine<TME> {
    fn default() -> Self {
        Self {
            text_measurement_engine: TME::default(),
        }
    }
}

impl<TME: TextMeasurementEngine + Clone> Clone for DefaultLayoutEngine<TME> {
    fn clone(&self) -> Self {
        Self {
            text_measurement_engine: self.text_measurement_engine.clone(),
        }
    }
}

impl<TME: TextMeasurementEngine + Copy> Copy for DefaultLayoutEngine<TME> {}

impl<TME: TextMeasurementEngine> DefaultLayoutEngine<TME> {
    pub fn new(engine: TME) -> Self {
        Self {
            text_measurement_engine: engine,
        }
    }

    pub fn layout_node(
        &self,
        size_available: Vec2,
        mapping: &CoordsMapping,
        unit: &WidgetUnit,
    ) -> Option<LayoutNode> {
        match unit {
            WidgetUnit::None | WidgetUnit::PortalBox(_) => None,
            WidgetUnit::AreaBox(b) => self.layout_area_box(size_available, mapping, b),
            WidgetUnit::ContentBox(b) => self.layout_content_box(size_available, mapping, b),
            WidgetUnit::FlexBox(b) => self.layout_flex_box(size_available, mapping, b),
            WidgetUnit::GridBox(b) => self.layout_grid_box(size_available, mapping, b),
            WidgetUnit::SizeBox(b) => self.layout_size_box(size_available, mapping, b),
            WidgetUnit::ImageBox(b) => self.layout_image_box(size_available, b),
            WidgetUnit::TextBox(b) => self.layout_text_box(size_available, mapping, b),
        }
    }

    pub fn layout_area_box(
        &self,
        size_available: Vec2,
        mapping: &CoordsMapping,
        unit: &AreaBox,
    ) -> Option<LayoutNode> {
        if !unit.id.is_valid() {
            return None;
        }
        let (children, w, h) =
            if let Some(child) = self.layout_node(size_available, mapping, &unit.slot) {
                let w = child.local_space.width();
                let h = child.local_space.height();
                (vec![child], w, h)
            } else {
                (vec![], 0.0, 0.0)
            };
        let local_space = Rect {
            left: 0.0,
            right: w,
            top: 0.0,
            bottom: h,
        };
        Some(LayoutNode {
            id: unit.id.to_owned(),
            local_space,
            children,
        })
    }

    pub fn layout_content_box(
        &self,
        size_available: Vec2,
        mapping: &CoordsMapping,
        unit: &ContentBox,
    ) -> Option<LayoutNode> {
        if !unit.id.is_valid() {
            return None;
        }
        let children = unit
            .items
            .iter()
            .filter_map(|item| {
                let left = lerp(0.0, size_available.x, item.layout.anchors.left);
                let left = left + item.layout.margin.left + item.layout.offset.x;
                let right = lerp(0.0, size_available.x, item.layout.anchors.right);
                let right = right - item.layout.margin.right + item.layout.offset.x;
                let top = lerp(0.0, size_available.y, item.layout.anchors.top);
                let top = top + item.layout.margin.top + item.layout.offset.y;
                let bottom = lerp(0.0, size_available.y, item.layout.anchors.bottom);
                let bottom = bottom - item.layout.margin.bottom + item.layout.offset.y;
                let width = (right - left).max(0.0);
                let height = (bottom - top).max(0.0);
                let size = Vec2 {
                    x: width,
                    y: height,
                };
                if let Some(mut child) = self.layout_node(size, mapping, &item.slot) {
                    let diff = child.local_space.width() - width;
                    let ox = lerp(0.0, diff, item.layout.align.x);
                    child.local_space.left += left - ox;
                    child.local_space.right += left - ox;
                    let diff = child.local_space.height() - height;
                    let oy = lerp(0.0, diff, item.layout.align.y);
                    child.local_space.top += top - oy;
                    child.local_space.bottom += top - oy;
                    let w = child.local_space.width().min(size_available.x);
                    let h = child.local_space.height().min(size_available.y);
                    if item.layout.keep_in_bounds.cut.left {
                        child.local_space.left = child.local_space.left.max(0.0);
                        if item.layout.keep_in_bounds.preserve.width {
                            child.local_space.right = child.local_space.left + w;
                        }
                    }
                    if item.layout.keep_in_bounds.cut.right {
                        child.local_space.right = child.local_space.right.min(size_available.x);
                        if item.layout.keep_in_bounds.preserve.width {
                            child.local_space.left = child.local_space.right - w;
                        }
                    }
                    if item.layout.keep_in_bounds.cut.top {
                        child.local_space.top = child.local_space.top.max(0.0);
                        if item.layout.keep_in_bounds.preserve.height {
                            child.local_space.bottom = child.local_space.top + h;
                        }
                    }
                    if item.layout.keep_in_bounds.cut.bottom {
                        child.local_space.bottom = child.local_space.bottom.min(size_available.y);
                        if item.layout.keep_in_bounds.preserve.height {
                            child.local_space.top = child.local_space.bottom - h;
                        }
                    }
                    Some(child)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        Some(LayoutNode {
            id: unit.id.to_owned(),
            local_space: Rect {
                left: 0.0,
                right: size_available.x,
                top: 0.0,
                bottom: size_available.y,
            },
            children,
        })
    }

    pub fn layout_flex_box(
        &self,
        size_available: Vec2,
        mapping: &CoordsMapping,
        unit: &FlexBox,
    ) -> Option<LayoutNode> {
        if !unit.id.is_valid() {
            return None;
        }
        if unit.wrap {
            Some(self.layout_flex_box_wrapping(size_available, mapping, unit))
        } else {
            Some(self.layout_flex_box_no_wrap(size_available, mapping, unit))
        }
    }

    pub fn layout_flex_box_wrapping(
        &self,
        size_available: Vec2,
        mapping: &CoordsMapping,
        unit: &FlexBox,
    ) -> LayoutNode {
        let main_available = if unit.direction.is_horizontal() {
            size_available.x
        } else {
            size_available.y
        };
        let (lines, count) = {
            let mut main = 0.0;
            let mut cross: Scalar = 0.0;
            let mut grow = 0.0;
            let items = unit
                .items
                .iter()
                .filter(|item| item.slot.is_some() && item.slot.as_data().unwrap().id().is_valid())
                .collect::<Vec<_>>();
            let count = items.len();
            let mut lines = vec![];
            let mut line = vec![];
            for item in items {
                let local_main = item.layout.basis.unwrap_or_else(|| {
                    if unit.direction.is_horizontal() {
                        self.calc_unit_min_width(size_available, mapping, &item.slot)
                    } else {
                        self.calc_unit_min_height(size_available, mapping, &item.slot)
                    }
                });
                let local_main = local_main
                    + if unit.direction.is_horizontal() {
                        item.layout.margin.left + item.layout.margin.right
                    } else {
                        item.layout.margin.top + item.layout.margin.bottom
                    };
                let local_cross = if unit.direction.is_horizontal() {
                    self.calc_unit_min_height(size_available, mapping, &item.slot)
                } else {
                    self.calc_unit_min_width(size_available, mapping, &item.slot)
                };
                let local_cross = local_cross
                    + if unit.direction.is_horizontal() {
                        item.layout.margin.top + item.layout.margin.bottom
                    } else {
                        item.layout.margin.left + item.layout.margin.right
                    };
                if !line.is_empty() && main + local_main > main_available {
                    main += line.len().saturating_sub(1) as Scalar * unit.separation;
                    lines.push((main, cross, grow, std::mem::take(&mut line)));
                    main = 0.0;
                    cross = 0.0;
                    grow = 0.0;
                }
                main += local_main;
                cross = cross.max(local_cross);
                grow += item.layout.grow;
                line.push((item, local_main, local_cross));
            }
            main += line.len().saturating_sub(1) as Scalar * unit.separation;
            lines.push((main, cross, grow, line));
            (lines, count)
        };
        let mut children = Vec::with_capacity(count);
        let mut main_max: Scalar = 0.0;
        let mut cross_max = 0.0;
        for (main, cross_available, grow, items) in lines {
            let diff = main_available - main;
            let mut new_main = 0.0;
            let mut new_cross: Scalar = 0.0;
            for (item, local_main, local_cross) in items {
                let child_main = if main < main_available {
                    local_main
                        + if grow > 0.0 {
                            diff * item.layout.grow / grow
                        } else {
                            0.0
                        }
                } else {
                    local_main
                };
                let child_main = (child_main
                    - if unit.direction.is_horizontal() {
                        item.layout.margin.left + item.layout.margin.right
                    } else {
                        item.layout.margin.top + item.layout.margin.bottom
                    })
                .max(0.0);
                let child_cross = (local_cross
                    - if unit.direction.is_horizontal() {
                        item.layout.margin.top + item.layout.margin.bottom
                    } else {
                        item.layout.margin.left + item.layout.margin.right
                    })
                .max(0.0);
                let child_cross = lerp(child_cross, cross_available, item.layout.fill);
                let rect = if unit.direction.is_horizontal() {
                    Vec2 {
                        x: child_main,
                        y: child_cross,
                    }
                } else {
                    Vec2 {
                        x: child_cross,
                        y: child_main,
                    }
                };
                if let Some(mut child) = self.layout_node(rect, mapping, &item.slot) {
                    if unit.direction.is_horizontal() {
                        if unit.direction.is_order_ascending() {
                            child.local_space.left += new_main + item.layout.margin.left;
                            child.local_space.right += new_main + item.layout.margin.left;
                        } else {
                            let left = child.local_space.left;
                            let right = child.local_space.right;
                            child.local_space.left =
                                size_available.x - right - new_main - item.layout.margin.right;
                            child.local_space.right =
                                size_available.x - left - new_main - item.layout.margin.right;
                        }
                        new_main += rect.x + item.layout.margin.left + item.layout.margin.right;
                        let diff = lerp(
                            0.0,
                            cross_available - child.local_space.height(),
                            item.layout.align,
                        );
                        child.local_space.top += cross_max + item.layout.margin.top + diff;
                        child.local_space.bottom += cross_max + item.layout.margin.top + diff;
                        new_cross = new_cross.max(rect.y);
                    } else {
                        if unit.direction.is_order_ascending() {
                            child.local_space.top += new_main + item.layout.margin.top;
                            child.local_space.bottom += new_main + item.layout.margin.top;
                        } else {
                            let top = child.local_space.top;
                            let bottom = child.local_space.bottom;
                            child.local_space.top =
                                size_available.y - bottom - new_main - item.layout.margin.bottom;
                            child.local_space.bottom =
                                size_available.y - top - new_main - item.layout.margin.bottom;
                        }
                        new_main += rect.y + item.layout.margin.top + item.layout.margin.bottom;
                        let diff = lerp(
                            0.0,
                            cross_available - child.local_space.width(),
                            item.layout.align,
                        );
                        child.local_space.left += cross_max + item.layout.margin.left + diff;
                        child.local_space.right += cross_max + item.layout.margin.left + diff;
                        new_cross = new_cross.max(rect.x);
                    }
                    new_main += unit.separation;
                    children.push(child);
                }
            }
            new_main = (new_main - unit.separation).max(0.0);
            main_max = main_max.max(new_main);
            cross_max += new_cross + unit.separation;
        }
        cross_max = (cross_max - unit.separation).max(0.0);
        let local_space = if unit.direction.is_horizontal() {
            Rect {
                left: 0.0,
                right: main_max,
                top: 0.0,
                bottom: cross_max,
            }
        } else {
            Rect {
                left: 0.0,
                right: cross_max,
                top: 0.0,
                bottom: main_max,
            }
        };
        LayoutNode {
            id: unit.id.to_owned(),
            local_space,
            children,
        }
    }

    pub fn layout_flex_box_no_wrap(
        &self,
        size_available: Vec2,
        mapping: &CoordsMapping,
        unit: &FlexBox,
    ) -> LayoutNode {
        let (main_available, cross_available) = if unit.direction.is_horizontal() {
            (size_available.x, size_available.y)
        } else {
            (size_available.y, size_available.x)
        };
        let mut main = 0.0;
        let mut cross: Scalar = 0.0;
        let mut grow = 0.0;
        let mut shrink = 0.0;
        let items = unit
            .items
            .iter()
            .filter(|item| item.slot.is_some() && item.slot.as_data().unwrap().id().is_valid())
            .collect::<Vec<_>>();
        let mut axis_sizes = Vec::with_capacity(items.len());
        for item in &items {
            let local_main = item.layout.basis.unwrap_or_else(|| {
                if unit.direction.is_horizontal() {
                    self.calc_unit_min_width(size_available, mapping, &item.slot)
                } else {
                    self.calc_unit_min_height(size_available, mapping, &item.slot)
                }
            });
            let local_main = local_main
                + if unit.direction.is_horizontal() {
                    item.layout.margin.left + item.layout.margin.right
                } else {
                    item.layout.margin.top + item.layout.margin.bottom
                };
            let local_cross = if unit.direction.is_horizontal() {
                self.calc_unit_min_height(size_available, mapping, &item.slot)
            } else {
                self.calc_unit_min_width(size_available, mapping, &item.slot)
            };
            let local_cross = local_cross
                + if unit.direction.is_horizontal() {
                    item.layout.margin.top + item.layout.margin.bottom
                } else {
                    item.layout.margin.left + item.layout.margin.right
                };
            let local_cross = lerp(local_cross, cross_available, item.layout.fill);
            main += local_main;
            cross = cross.max(local_cross);
            grow += item.layout.grow;
            shrink += item.layout.shrink;
            axis_sizes.push((local_main, local_cross));
        }
        main += items.len().saturating_sub(1) as Scalar * unit.separation;
        let diff = main_available - main;
        let mut new_main = 0.0;
        let mut new_cross: Scalar = 0.0;
        let children = items
            .into_iter()
            .zip(axis_sizes)
            .filter_map(|(item, axis_size)| {
                let child_main = if main < main_available {
                    axis_size.0
                        + if grow > 0.0 {
                            diff * item.layout.grow / grow
                        } else {
                            0.0
                        }
                } else if main > main_available {
                    axis_size.0
                        + if shrink > 0.0 {
                            diff * item.layout.shrink / shrink
                        } else {
                            0.0
                        }
                } else {
                    axis_size.0
                };
                let child_main = (child_main
                    - if unit.direction.is_horizontal() {
                        item.layout.margin.left + item.layout.margin.right
                    } else {
                        item.layout.margin.top + item.layout.margin.bottom
                    })
                .max(0.0);
                let child_cross = (axis_size.1
                    - if unit.direction.is_horizontal() {
                        item.layout.margin.top + item.layout.margin.bottom
                    } else {
                        item.layout.margin.left + item.layout.margin.right
                    })
                .max(0.0);
                let rect = if unit.direction.is_horizontal() {
                    Vec2 {
                        x: child_main,
                        y: child_cross,
                    }
                } else {
                    Vec2 {
                        x: child_cross,
                        y: child_main,
                    }
                };
                if let Some(mut child) = self.layout_node(rect, mapping, &item.slot) {
                    if unit.direction.is_horizontal() {
                        if unit.direction.is_order_ascending() {
                            child.local_space.left += new_main + item.layout.margin.left;
                            child.local_space.right += new_main + item.layout.margin.left;
                        } else {
                            let left = child.local_space.left;
                            let right = child.local_space.right;
                            child.local_space.left =
                                size_available.x - right - new_main - item.layout.margin.right;
                            child.local_space.right =
                                size_available.x - left - new_main - item.layout.margin.right;
                        }
                        new_main += rect.x + item.layout.margin.left + item.layout.margin.right;
                        let diff = lerp(
                            0.0,
                            cross_available - child.local_space.height(),
                            item.layout.align,
                        );
                        child.local_space.top += item.layout.margin.top + diff;
                        child.local_space.bottom += item.layout.margin.top + diff;
                        new_cross = new_cross.max(rect.y);
                    } else {
                        if unit.direction.is_order_ascending() {
                            child.local_space.top += new_main + item.layout.margin.top;
                            child.local_space.bottom += new_main + item.layout.margin.top;
                        } else {
                            let top = child.local_space.top;
                            let bottom = child.local_space.bottom;
                            child.local_space.top =
                                size_available.y - bottom - new_main - item.layout.margin.bottom;
                            child.local_space.bottom =
                                size_available.y - top - new_main - item.layout.margin.bottom;
                        }
                        new_main += rect.y + item.layout.margin.top + item.layout.margin.bottom;
                        let diff = lerp(
                            0.0,
                            cross_available - child.local_space.width(),
                            item.layout.align,
                        );
                        child.local_space.left += item.layout.margin.left + diff;
                        child.local_space.right += item.layout.margin.left + diff;
                        new_cross = new_cross.max(rect.x);
                    }
                    new_main += unit.separation;
                    Some(child)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        new_main = (new_main - unit.separation).max(0.0);
        let local_space = if unit.direction.is_horizontal() {
            Rect {
                left: 0.0,
                right: new_main,
                top: 0.0,
                bottom: new_cross,
            }
        } else {
            Rect {
                left: 0.0,
                right: new_cross,
                top: 0.0,
                bottom: new_main,
            }
        };
        LayoutNode {
            id: unit.id.to_owned(),
            local_space,
            children,
        }
    }

    pub fn layout_grid_box(
        &self,
        size_available: Vec2,
        mapping: &CoordsMapping,
        unit: &GridBox,
    ) -> Option<LayoutNode> {
        if !unit.id.is_valid() {
            return None;
        }
        let cell_width = if unit.cols > 0 {
            size_available.x / unit.cols as Scalar
        } else {
            0.0
        };
        let cell_height = if unit.rows > 0 {
            size_available.y / unit.rows as Scalar
        } else {
            0.0
        };
        let children = unit
            .items
            .iter()
            .filter_map(|item| {
                let left = item.layout.space_occupancy.left as Scalar * cell_width;
                let right = item.layout.space_occupancy.right as Scalar * cell_width;
                let top = item.layout.space_occupancy.top as Scalar * cell_height;
                let bottom = item.layout.space_occupancy.bottom as Scalar * cell_height;
                let width =
                    (right - left - item.layout.margin.left - item.layout.margin.right).max(0.0);
                let height =
                    (bottom - top - item.layout.margin.top - item.layout.margin.bottom).max(0.0);
                let size = Vec2 {
                    x: width,
                    y: height,
                };
                if let Some(mut child) = self.layout_node(size, mapping, &item.slot) {
                    let diff = size.x - child.local_space.width();
                    let ox = lerp(0.0, diff, item.layout.horizontal_align);
                    let diff = size.y - child.local_space.height();
                    let oy = lerp(0.0, diff, item.layout.vertical_align);
                    child.local_space.left += left + item.layout.margin.left - ox;
                    child.local_space.right += left + item.layout.margin.left - ox;
                    child.local_space.top += top + item.layout.margin.top - oy;
                    child.local_space.bottom += top + item.layout.margin.top - oy;
                    Some(child)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        Some(LayoutNode {
            id: unit.id.to_owned(),
            local_space: Rect {
                left: 0.0,
                right: size_available.x,
                top: 0.0,
                bottom: size_available.y,
            },
            children,
        })
    }

    pub fn layout_size_box(
        &self,
        size_available: Vec2,
        mapping: &CoordsMapping,
        unit: &SizeBox,
    ) -> Option<LayoutNode> {
        if !unit.id.is_valid() {
            return None;
        }
        let mut size = Vec2 {
            x: match unit.width {
                SizeBoxSizeValue::Content => {
                    self.calc_unit_min_width(size_available, mapping, &unit.slot)
                }
                SizeBoxSizeValue::Fill => size_available.x - unit.margin.left - unit.margin.right,
                SizeBoxSizeValue::Exact(v) => v,
            },
            y: match unit.height {
                SizeBoxSizeValue::Content => {
                    self.calc_unit_min_height(size_available, mapping, &unit.slot)
                }
                SizeBoxSizeValue::Fill => size_available.y - unit.margin.top - unit.margin.bottom,
                SizeBoxSizeValue::Exact(v) => v,
            },
        };
        match unit.keep_aspect_ratio {
            SizeBoxAspectRatio::None => {}
            SizeBoxAspectRatio::WidthOfHeight(factor) => {
                size.x = (size.y * factor).max(0.0);
            }
            SizeBoxAspectRatio::HeightOfWidth(factor) => {
                size.y = (size.x * factor).max(0.0);
            }
        }
        let children = if let Some(mut child) = self.layout_node(size, mapping, &unit.slot) {
            child.local_space.left += unit.margin.left;
            child.local_space.right += unit.margin.left;
            child.local_space.top += unit.margin.top;
            child.local_space.bottom += unit.margin.top;
            vec![child]
        } else {
            vec![]
        };
        let local_space = Rect {
            left: 0.0,
            right: size.x,
            top: 0.0,
            bottom: size.y,
        };
        Some(LayoutNode {
            id: unit.id.to_owned(),
            local_space,
            children,
        })
    }

    pub fn layout_image_box(&self, size_available: Vec2, unit: &ImageBox) -> Option<LayoutNode> {
        if !unit.id.is_valid() {
            return None;
        }
        let local_space = Rect {
            left: 0.0,
            right: match unit.width {
                ImageBoxSizeValue::Fill => size_available.x,
                ImageBoxSizeValue::Exact(v) => v,
            },
            top: 0.0,
            bottom: match unit.height {
                ImageBoxSizeValue::Fill => size_available.y,
                ImageBoxSizeValue::Exact(v) => v,
            },
        };
        Some(LayoutNode {
            id: unit.id.to_owned(),
            local_space,
            children: vec![],
        })
    }

    pub fn layout_text_box(
        &self,
        size_available: Vec2,
        mapping: &CoordsMapping,
        unit: &TextBox,
    ) -> Option<LayoutNode> {
        if !unit.id.is_valid() {
            return None;
        }
        let aabb = self
            .text_measurement_engine
            .measure_text(size_available, mapping, unit)
            .unwrap_or_default();
        let local_space = Rect {
            left: 0.0,
            right: match unit.width {
                TextBoxSizeValue::Content => aabb.width(),
                TextBoxSizeValue::Fill => size_available.x,
                TextBoxSizeValue::Exact(v) => v,
            },
            top: 0.0,
            bottom: match unit.height {
                TextBoxSizeValue::Content => aabb.height(),
                TextBoxSizeValue::Fill => size_available.y,
                TextBoxSizeValue::Exact(v) => v,
            },
        };
        Some(LayoutNode {
            id: unit.id.to_owned(),
            local_space,
            children: vec![],
        })
    }

    fn calc_unit_min_width(
        &self,
        size_available: Vec2,
        mapping: &CoordsMapping,
        unit: &WidgetUnit,
    ) -> Scalar {
        match unit {
            WidgetUnit::None | WidgetUnit::PortalBox(_) => 0.0,
            WidgetUnit::AreaBox(b) => self.calc_unit_min_width(size_available, mapping, &b.slot),
            WidgetUnit::ContentBox(b) => {
                self.calc_content_box_min_width(size_available, mapping, b)
            }
            WidgetUnit::FlexBox(b) => self.calc_flex_box_min_width(size_available, mapping, b),
            WidgetUnit::GridBox(b) => self.calc_grid_box_min_width(size_available, mapping, b),
            WidgetUnit::SizeBox(b) => {
                (match b.width {
                    SizeBoxSizeValue::Content => {
                        self.calc_unit_min_width(size_available, mapping, &b.slot)
                    }
                    SizeBoxSizeValue::Fill => 0.0,
                    SizeBoxSizeValue::Exact(v) => v,
                }) + b.margin.left
                    + b.margin.right
            }
            WidgetUnit::ImageBox(b) => match b.width {
                ImageBoxSizeValue::Fill => 0.0,
                ImageBoxSizeValue::Exact(v) => v,
            },
            WidgetUnit::TextBox(b) => {
                let aabb = self
                    .text_measurement_engine
                    .measure_text(size_available, mapping, b)
                    .unwrap_or_default();
                match b.width {
                    TextBoxSizeValue::Content => aabb.width(),
                    TextBoxSizeValue::Fill => 0.0,
                    TextBoxSizeValue::Exact(v) => v,
                }
            }
        }
    }

    fn calc_content_box_min_width(
        &self,
        size_available: Vec2,
        mapping: &CoordsMapping,
        unit: &ContentBox,
    ) -> Scalar {
        let mut result: Scalar = 0.0;
        for item in &unit.items {
            let size = self.calc_unit_min_width(size_available, mapping, &item.slot)
                + item.layout.margin.left
                + item.layout.margin.right;
            let width = item.layout.anchors.right - item.layout.anchors.left;
            let size = if width > 0.0 { size / width } else { 0.0 };
            result = result.max(size);
        }
        result
    }

    fn calc_flex_box_min_width(
        &self,
        size_available: Vec2,
        mapping: &CoordsMapping,
        unit: &FlexBox,
    ) -> Scalar {
        if unit.direction.is_horizontal() {
            self.calc_horizontal_flex_box_min_width(size_available, mapping, unit)
        } else {
            self.calc_vertical_flex_box_min_width(size_available, mapping, unit)
        }
    }

    fn calc_horizontal_flex_box_min_width(
        &self,
        size_available: Vec2,
        mapping: &CoordsMapping,
        unit: &FlexBox,
    ) -> Scalar {
        if unit.wrap {
            let mut result: Scalar = 0.0;
            let mut line = 0.0;
            let mut first = true;
            for item in &unit.items {
                let size = self.calc_unit_min_width(size_available, mapping, &item.slot)
                    + item.layout.margin.left
                    + item.layout.margin.right;
                if first || line + size <= size_available.x {
                    line += size;
                    if !first {
                        line += unit.separation;
                    }
                    first = false;
                } else {
                    result = result.max(line);
                    line = 0.0;
                    first = true;
                }
            }
            result.max(line)
        } else {
            let mut result = 0.0;
            for item in &unit.items {
                result += self.calc_unit_min_width(size_available, mapping, &item.slot)
                    + item.layout.margin.left
                    + item.layout.margin.right;
            }
            result + (unit.items.len().saturating_sub(1) as Scalar) * unit.separation
        }
    }

    fn calc_vertical_flex_box_min_width(
        &self,
        size_available: Vec2,
        mapping: &CoordsMapping,
        unit: &FlexBox,
    ) -> Scalar {
        if unit.wrap {
            let mut result = 0.0;
            let mut line_length = 0.0;
            let mut line: Scalar = 0.0;
            let mut lines: usize = 0;
            let mut first = true;
            for item in &unit.items {
                let width = self.calc_unit_min_width(size_available, mapping, &item.slot)
                    + item.layout.margin.left
                    + item.layout.margin.right;
                let height = self.calc_unit_min_height(size_available, mapping, &item.slot)
                    + item.layout.margin.top
                    + item.layout.margin.bottom;
                if first || line_length + height <= size_available.y {
                    line_length += height;
                    if !first {
                        line_length += unit.separation;
                    }
                    line = line.max(width);
                    first = false;
                } else {
                    result += line;
                    line_length = 0.0;
                    line = 0.0;
                    lines += 1;
                    first = true;
                }
            }
            result += line;
            lines += 1;
            result + (lines.saturating_sub(1) as Scalar) * unit.separation
        } else {
            unit.items.iter().fold(0.0, |a, item| {
                (self.calc_unit_min_width(size_available, mapping, &item.slot)
                    + item.layout.margin.left
                    + item.layout.margin.right)
                    .max(a)
            })
        }
    }

    fn calc_grid_box_min_width(
        &self,
        size_available: Vec2,
        mapping: &CoordsMapping,
        unit: &GridBox,
    ) -> Scalar {
        let mut result: Scalar = 0.0;
        for item in &unit.items {
            let size = self.calc_unit_min_width(size_available, mapping, &item.slot)
                + item.layout.margin.left
                + item.layout.margin.right;
            let size = if size > 0.0 {
                (item.layout.space_occupancy.width() as Scalar * size) / unit.cols as Scalar
            } else {
                0.0
            };
            result = result.max(size);
        }
        result
    }

    fn calc_unit_min_height(
        &self,
        size_available: Vec2,
        mapping: &CoordsMapping,
        unit: &WidgetUnit,
    ) -> Scalar {
        match unit {
            WidgetUnit::None | WidgetUnit::PortalBox(_) => 0.0,
            WidgetUnit::AreaBox(b) => self.calc_unit_min_height(size_available, mapping, &b.slot),
            WidgetUnit::ContentBox(b) => {
                self.calc_content_box_min_height(size_available, mapping, b)
            }
            WidgetUnit::FlexBox(b) => self.calc_flex_box_min_height(size_available, mapping, b),
            WidgetUnit::GridBox(b) => self.calc_grid_box_min_height(size_available, mapping, b),
            WidgetUnit::SizeBox(b) => {
                (match b.height {
                    SizeBoxSizeValue::Content => {
                        self.calc_unit_min_height(size_available, mapping, &b.slot)
                    }
                    SizeBoxSizeValue::Fill => 0.0,
                    SizeBoxSizeValue::Exact(v) => v,
                }) + b.margin.top
                    + b.margin.bottom
            }
            WidgetUnit::ImageBox(b) => match b.height {
                ImageBoxSizeValue::Fill => 0.0,
                ImageBoxSizeValue::Exact(v) => v,
            },
            WidgetUnit::TextBox(b) => {
                let aabb = self
                    .text_measurement_engine
                    .measure_text(size_available, mapping, b)
                    .unwrap_or_default();
                match b.height {
                    TextBoxSizeValue::Content => aabb.height(),
                    TextBoxSizeValue::Fill => 0.0,
                    TextBoxSizeValue::Exact(v) => v,
                }
            }
        }
    }

    fn calc_content_box_min_height(
        &self,
        size_available: Vec2,
        mapping: &CoordsMapping,
        unit: &ContentBox,
    ) -> Scalar {
        let mut result: Scalar = 0.0;
        for item in &unit.items {
            let size = self.calc_unit_min_height(size_available, mapping, &item.slot)
                + item.layout.margin.top
                + item.layout.margin.bottom;
            let height = item.layout.anchors.bottom - item.layout.anchors.top;
            let size = if height > 0.0 { size / height } else { 0.0 };
            result = result.max(size);
        }
        result
    }

    fn calc_flex_box_min_height(
        &self,
        size_available: Vec2,
        mapping: &CoordsMapping,
        unit: &FlexBox,
    ) -> Scalar {
        if unit.direction.is_horizontal() {
            self.calc_horizontal_flex_box_min_height(size_available, mapping, unit)
        } else {
            self.calc_vertical_flex_box_min_height(size_available, mapping, unit)
        }
    }

    fn calc_horizontal_flex_box_min_height(
        &self,
        size_available: Vec2,
        mapping: &CoordsMapping,
        unit: &FlexBox,
    ) -> Scalar {
        if unit.wrap {
            let mut result = 0.0;
            let mut line_length = 0.0;
            let mut line: Scalar = 0.0;
            let mut lines: usize = 0;
            let mut first = true;
            for item in &unit.items {
                let width = self.calc_unit_min_width(size_available, mapping, &item.slot)
                    + item.layout.margin.left
                    + item.layout.margin.right;
                let height = self.calc_unit_min_height(size_available, mapping, &item.slot)
                    + item.layout.margin.top
                    + item.layout.margin.bottom;
                if first || line_length + width <= size_available.x {
                    line_length += width;
                    if !first {
                        line_length += unit.separation;
                    }
                    line = line.max(height);
                    first = false;
                } else {
                    result += line;
                    line_length = 0.0;
                    line = 0.0;
                    lines += 1;
                    first = true;
                }
            }
            result += line;
            lines += 1;
            result + (lines.saturating_sub(1) as Scalar) * unit.separation
        } else {
            unit.items.iter().fold(0.0, |a, item| {
                (self.calc_unit_min_height(size_available, mapping, &item.slot)
                    + item.layout.margin.top
                    + item.layout.margin.bottom)
                    .max(a)
            })
        }
    }

    fn calc_vertical_flex_box_min_height(
        &self,
        size_available: Vec2,
        mapping: &CoordsMapping,
        unit: &FlexBox,
    ) -> Scalar {
        if unit.wrap {
            let mut result: Scalar = 0.0;
            let mut line = 0.0;
            let mut first = true;
            for item in &unit.items {
                let size = self.calc_unit_min_height(size_available, mapping, &item.slot)
                    + item.layout.margin.top
                    + item.layout.margin.bottom;
                if first || line + size <= size_available.y {
                    line += size;
                    if !first {
                        line += unit.separation;
                    }
                    first = false;
                } else {
                    result = result.max(line);
                    line = 0.0;
                    first = true;
                }
            }
            result.max(line)
        } else {
            let mut result = 0.0;
            for item in &unit.items {
                result += self.calc_unit_min_height(size_available, mapping, &item.slot)
                    + item.layout.margin.top
                    + item.layout.margin.bottom;
            }
            result + (unit.items.len().saturating_sub(1) as Scalar) * unit.separation
        }
    }

    fn calc_grid_box_min_height(
        &self,
        size_available: Vec2,
        mapping: &CoordsMapping,
        unit: &GridBox,
    ) -> Scalar {
        let mut result: Scalar = 0.0;
        for item in &unit.items {
            let size = self.calc_unit_min_height(size_available, mapping, &item.slot)
                + item.layout.margin.top
                + item.layout.margin.bottom;
            let size = if size > 0.0 {
                (item.layout.space_occupancy.height() as Scalar * size) / unit.cols as Scalar
            } else {
                0.0
            };
            result = result.max(size);
        }
        result
    }

    fn unpack_node(
        parent: Option<&WidgetId>,
        ui_space: Rect,
        node: LayoutNode,
        items: &mut HashMap<WidgetId, LayoutItem>,
    ) {
        let LayoutNode {
            id,
            local_space,
            children,
        } = node;
        let ui_space = Rect {
            left: local_space.left + ui_space.left,
            right: local_space.right + ui_space.left,
            top: local_space.top + ui_space.top,
            bottom: local_space.bottom + ui_space.top,
        };
        for node in children {
            Self::unpack_node(Some(&id), ui_space, node, items);
        }
        items.insert(
            id,
            LayoutItem {
                local_space,
                ui_space,
                parent: parent.cloned(),
            },
        );
    }
}

impl<TME: TextMeasurementEngine> LayoutEngine<()> for DefaultLayoutEngine<TME> {
    fn layout(&mut self, mapping: &CoordsMapping, tree: &WidgetUnit) -> Result<Layout, ()> {
        let ui_space = mapping.virtual_area();
        if let Some(root) = self.layout_node(ui_space.size(), mapping, tree) {
            let mut items = HashMap::with_capacity(root.count());
            Self::unpack_node(None, ui_space, root, &mut items);
            Ok(Layout { ui_space, items })
        } else {
            Ok(Layout {
                ui_space,
                items: Default::default(),
            })
        }
    }
}
