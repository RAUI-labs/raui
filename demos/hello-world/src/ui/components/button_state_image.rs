use crate::ui::components::image_button::ImageButtonProps;
use raui_core::prelude::*;

widget_component! {
    pub button_state_image(key, props) {
        let ButtonProps {
            selected,
            trigger,
            context,
            ..
        } = props.read_cloned_or_default();
        let ImageButtonProps {
            id,
            horizontal_alignment,
        } = props.read_cloned_or_default();
        let scale = if trigger || context {
            Vec2 {
                x: 1.1,
                y: 1.1,
            }
        } else if selected {
            Vec2 {
                x: 1.05,
                y: 1.05,
            }
        } else {
            Vec2 {
                x: 1.0,
                y: 1.0,
            }
        };
        let image_props = ImageBoxProps {
            material: ImageBoxMaterial::Image(ImageBoxImage {
                id,
                tint: if trigger {
                    Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 }
                } else if context {
                    Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 }
                } else if selected {
                    Color { r: 1.0, g: 1.0, b: 1.0, a: 0.85 }
                } else {
                    Color::default()
                },
                ..Default::default()
            }),
            content_keep_aspect_ratio: Some(ImageBoxAspectRatio {
                horizontal_alignment,
                vertical_alignment: 0.5
            }),
            transform: Transform {
                pivot: Vec2 {
                    x: 0.5,
                    y: 0.5,
                },
                scale,
                ..Default::default()
            },
            ..Default::default()
        };

        widget! {
            (#{key} image_box: {image_props})
        }
    }
}
