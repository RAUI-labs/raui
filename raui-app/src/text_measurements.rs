use crate::AssetsManager;
use fontdue::layout::{
    CoordinateSystem, HorizontalAlign, Layout, LayoutSettings, TextStyle, VerticalAlign,
};
use raui_core::{
    layout::{CoordsMapping, default_layout_engine::TextMeasurementEngine},
    widget::{
        unit::text::{TextBox, TextBoxHorizontalAlign, TextBoxSizeValue, TextBoxVerticalAlign},
        utils::{Rect, Vec2},
    },
};
use raui_tesselate_renderer::*;
use spitfire_fontdue::TextRenderer;

pub struct AppTextMeasurementsEngine<'a> {
    pub assets: &'a AssetsManager,
}

impl TextMeasurementEngine for AppTextMeasurementsEngine<'_> {
    fn measure_text(
        &self,
        size_available: Vec2,
        mapping: &CoordsMapping,
        unit: &TextBox,
    ) -> Option<Rect> {
        let font_index = self.assets.font_index_by_id(&unit.font.name)?;
        let text = TextStyle::with_user_data(
            &unit.text,
            unit.font.size * mapping.scalar_scale(false),
            font_index,
            unit.color,
        );
        let max_width = match unit.width {
            TextBoxSizeValue::Content => None,
            TextBoxSizeValue::Fill => Some(size_available.x),
            TextBoxSizeValue::Exact(v) => Some(v),
        };
        let max_height = match unit.height {
            TextBoxSizeValue::Content => None,
            TextBoxSizeValue::Fill => Some(size_available.y),
            TextBoxSizeValue::Exact(v) => Some(v),
        };
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(&LayoutSettings {
            max_width,
            max_height,
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
        layout.append(self.assets.fonts(), &text);
        let aabb = TextRenderer::measure(&layout, self.assets.fonts(), false);
        if aabb.iter().all(|v| v.is_finite()) {
            Some(Rect {
                left: aabb[0],
                top: aabb[1],
                right: aabb[2],
                bottom: aabb[3],
            })
        } else {
            None
        }
    }
}
