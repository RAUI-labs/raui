use raui_core::{
    layout::{CoordsMapping, Layout},
    renderer::Renderer,
    widget::unit::WidgetUnit,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct JsonRenderer {
    pub pretty: bool,
}

impl Renderer<String, serde_json::Error> for JsonRenderer {
    fn render(
        &mut self,
        tree: &WidgetUnit,
        _: &CoordsMapping,
        _layout: &Layout,
    ) -> Result<String, serde_json::Error> {
        if self.pretty {
            serde_json::to_string_pretty(tree)
        } else {
            serde_json::to_string(tree)
        }
    }
}

impl Renderer<serde_json::Value, serde_json::Error> for JsonRenderer {
    fn render(
        &mut self,
        tree: &WidgetUnit,
        _: &CoordsMapping,
        _: &Layout,
    ) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(tree)
    }
}
