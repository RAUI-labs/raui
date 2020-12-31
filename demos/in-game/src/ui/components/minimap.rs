use raui_core::prelude::*;
use raui_material::prelude::*;

widget_component! {
    pub minimap(key, props) {
        let size_props = props.clone().with(SizeBoxProps {
            width: SizeBoxSizeValue::Exact(64.0),
            height: SizeBoxSizeValue::Exact(64.0),
            ..Default::default()
        });
        let panel_props = props.clone().with(PaperProps {
            frame: None,
            ..Default::default()
        });
        let image_props = Props::new(ImageBoxProps {
            material: ImageBoxMaterial::Image(ImageBoxImage {
                id: "in-game-minimap".to_owned(),
                tint: Color {
                    r: 0.75,
                    g: 0.75,
                    b: 0.75,
                    a: 1.0,
                },
                ..Default::default()
            }),
            ..Default::default()
        }).with(ContentBoxItemLayout {
            margin: Rect {
                left: 7.0,
                right: 6.0,
                top: 7.0,
                bottom: 6.0,
            },
            ..Default::default()
        });

        widget! {
            (#{key} size_box: {size_props} {
                content = (#{"panel"} paper: {panel_props} [
                    (#{"image"} image_box: {image_props})
                ])
            })
        }
    }
}
