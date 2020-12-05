use raui_core::{renderer::Renderer, widget::unit::WidgetUnit};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RonRenderer {
    pub pretty: Option<PrettyConfig>,
}

impl Renderer<String, ron::error::Error> for RonRenderer {
    fn render(&mut self, tree: &WidgetUnit) -> Result<String, ron::error::Error> {
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
    fn render(&mut self, tree: &WidgetUnit) -> Result<ron::value::Value, ron::error::Error> {
        match ron::ser::to_string(tree) {
            Ok(s) => ron::value::Value::from_str(&s),
            Err(e) => Err(e),
        }
    }
}
