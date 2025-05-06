use raui::core::{
    Prefab, PropsData, make_widget,
    props::PropsData,
    widget::{
        component::image_box::{ImageBoxProps, image_box},
        context::WidgetContext,
        node::WidgetNode,
        unit::image::{ImageBoxColor, ImageBoxImageScaling, ImageBoxMaterial},
        utils::Color,
    },
};
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct ColorRectProps {
    #[serde(default)]
    pub color: Color,
}

pub fn color_rect(context: WidgetContext) -> WidgetNode {
    let WidgetContext { key, props, .. } = context;

    let color = props.read_cloned_or_default::<ColorRectProps>().color;

    make_widget!(image_box)
        .key(key)
        .merge_props(props.clone())
        .with_props(ImageBoxProps {
            material: ImageBoxMaterial::Color(ImageBoxColor {
                color,
                scaling: ImageBoxImageScaling::Frame((10.0, true).into()),
            }),
            ..Default::default()
        })
        .into()
}
