use crate::AssetsManager;
use fontdue::layout::{
    CoordinateSystem, HorizontalAlign, Layout, LayoutSettings, TextStyle, VerticalAlign,
};
use raui_core::prelude::*;
use raui_tesselate_renderer::*;
use spitfire_fontdue::TextRenderer;

pub struct AppTextMeasurementsEngine<'a> {
    pub assets: &'a AssetsManager,
}

impl TextMeasurementEngine for AppTextMeasurementsEngine<'_> {
    fn measure_text(&self, mapping: &CoordsMapping, unit: &TextBox) -> Option<Rect> {
        let font_index = self.assets.font_index_by_id(&unit.font.name)?;
        let text = TextStyle::with_user_data(
            &unit.text,
            unit.font.size * mapping.scalar_scale(false),
            font_index,
            unit.color,
        );
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(&LayoutSettings {
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
        let aabb = TextRenderer::measure(self.assets.fonts(), &layout);
        if aabb.iter().all(|v| v.is_finite()) {
            Some(Rect {
                left: aabb[0].min(0.0),
                top: aabb[1].min(0.0),
                right: aabb[2],
                bottom: aabb[3],
            })
        } else {
            None
        }
    }
}
