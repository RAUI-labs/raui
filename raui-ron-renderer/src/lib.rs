use raui_core::{
    layout::{CoordsMapping, Layout},
    renderer::Renderer,
    widget::unit::WidgetUnit,
};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RonRenderer {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pretty: Option<PrettyConfig>,
}

impl Renderer<String, ron::error::Error> for RonRenderer {
    fn render(
        &mut self,
        tree: &WidgetUnit,
        _: &CoordsMapping,
        _layout: &Layout,
    ) -> Result<String, ron::error::Error> {
        if let Some(config) = &self.pretty {
            ron::ser::to_string_pretty(tree, config.clone())
        } else {
            ron::ser::to_string(tree)
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct RonValueRenderer;

impl Renderer<ron::value::Value, ron::error::Error> for RonValueRenderer {
    fn render(
        &mut self,
        tree: &WidgetUnit,
        _: &CoordsMapping,
        _: &Layout,
    ) -> Result<ron::value::Value, ron::error::Error> {
        match ron::ser::to_string(tree) {
            Ok(s) => ron::value::Value::from_str(&s),
            Err(e) => Err(e),
        }
    }
}
