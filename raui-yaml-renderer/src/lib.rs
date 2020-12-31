use raui_core::{
    layout::{CoordsMapping, Layout},
    renderer::Renderer,
    widget::unit::WidgetUnit,
};

#[derive(Debug, Default, Copy, Clone)]
pub struct YamlRenderer;

impl Renderer<String, serde_yaml::Error> for YamlRenderer {
    fn render(
        &mut self,
        tree: &WidgetUnit,
        _: &CoordsMapping,
        _layout: &Layout,
    ) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(tree)
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct YamlValueRenderer;

impl Renderer<serde_yaml::Value, serde_yaml::Error> for YamlValueRenderer {
    fn render(
        &mut self,
        tree: &WidgetUnit,
        _: &CoordsMapping,
        _: &Layout,
    ) -> Result<serde_yaml::Value, serde_yaml::Error> {
        serde_yaml::to_value(tree)
    }
}
