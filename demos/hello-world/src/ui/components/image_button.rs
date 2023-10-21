use raui::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
pub struct ImageButtonProps {
    #[serde(default)]
    pub image: String,
    #[serde(default)]
    pub horizontal_alignment: Scalar,
}

#[pre_hooks(use_button_notified_state)]
pub fn image_button(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        key,
        props,
        state,
        ..
    } = context;

    let ImageButtonProps {
        image,
        horizontal_alignment,
    } = props.read_cloned_or_default();
    let ButtonProps {
        selected,
        trigger,
        context,
        ..
    } = state.read_cloned_or_default();
    let scale = if trigger || context {
        Vec2 { x: 1.1, y: 1.1 }
    } else if selected {
        Vec2 { x: 1.05, y: 1.05 }
    } else {
        Vec2 { x: 1.0, y: 1.0 }
    };

    make_widget!(button)
        .key(key)
        .with_props(NavItemActive)
        .with_props(ButtonNotifyProps(id.to_owned().into()))
        .named_slot(
            "content",
            make_widget!(image_box)
                .key("image")
                .with_props(ImageBoxProps {
                    material: ImageBoxMaterial::Image(ImageBoxImage {
                        id: image,
                        tint: if trigger {
                            Color {
                                r: 0.0,
                                g: 1.0,
                                b: 0.0,
                                a: 1.0,
                            }
                        } else if context {
                            Color {
                                r: 1.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }
                        } else if selected {
                            Color {
                                r: 1.0,
                                g: 1.0,
                                b: 1.0,
                                a: 0.85,
                            }
                        } else {
                            Color::default()
                        },
                        ..Default::default()
                    }),
                    content_keep_aspect_ratio: Some(ImageBoxAspectRatio {
                        horizontal_alignment,
                        vertical_alignment: 0.5,
                        outside: false,
                    }),
                    transform: Transform {
                        pivot: Vec2 { x: 0.5, y: 0.5 },
                        scale,
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        )
        .into()
}
