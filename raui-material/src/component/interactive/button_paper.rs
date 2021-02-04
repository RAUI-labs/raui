use crate::{
    component::containers::paper::PaperProps,
    theme::{ThemeColor, ThemeProps, ThemeVariant, ThemedImageMaterial, ThemedWidgetProps},
};
use raui_core::prelude::*;

widget_component! {
    pub button_paper_content(key, props, shared_props, named_slots) {
        unpack_named_slots!(named_slots => content);

        let button_props = props.read_cloned_or_default::<ButtonProps>();
        let paper_props = props.read_cloned_or_default::<PaperProps>();
        let themed_props = props.read_cloned_or_default::<ThemedWidgetProps>();

        let items = match themed_props.variant {
            ThemeVariant::ContentOnly => vec![content],
            ThemeVariant::Filled => {
                let button_background = shared_props
                    .map_or_default::<ThemeProps, _, _>(|props| {
                        if button_props.trigger || button_props.context {
                            props
                                .button_backgrounds
                                .get(&paper_props.variant)
                                .cloned()
                                .unwrap_or_default()
                                .trigger
                        } else if button_props.selected {
                            props
                                .button_backgrounds
                                .get(&paper_props.variant)
                                .cloned()
                                .unwrap_or_default()
                                .selected
                        } else {
                            props
                                .button_backgrounds
                                .get(&paper_props.variant)
                                .cloned()
                                .unwrap_or_default()
                                .default
                        }
                    });
                let button_colors = shared_props.map_or_default::<ThemeProps, _, _>(|props| {
                    props.active_colors.clone()
                });
                let image = match button_background {
                    ThemedImageMaterial::Color => {
                        let color = match themed_props.color {
                            ThemeColor::Default => button_colors.main.default.main,
                            ThemeColor::Primary => button_colors.main.primary.main,
                            ThemeColor::Secondary => button_colors.main.secondary.main,
                        };
                        ImageBoxProps {
                            material: ImageBoxMaterial::Color(ImageBoxColor {
                                color,
                                ..Default::default()
                            }),
                            ..Default::default()
                        }
                    }
                    ThemedImageMaterial::Image(material) => {
                        ImageBoxProps {
                            material: ImageBoxMaterial::Image(material),
                            ..Default::default()
                        }
                    }
                    ThemedImageMaterial::Procedural(material) => {
                        ImageBoxProps {
                            material: ImageBoxMaterial::Procedural(material),
                            ..Default::default()
                        }
                    }
                };
                let props = Props::new(ContentBoxItemLayout {
                    depth: Scalar::NEG_INFINITY,
                    ..Default::default()
                }).with(image);
                let background = widget! {
                    (#{"background"} image_box: {props})
                };
                if let Some(frame) = paper_props.frame {
                    let color = match themed_props.color {
                        ThemeColor::Default => button_colors.main.default.dark,
                        ThemeColor::Primary => button_colors.main.primary.dark,
                        ThemeColor::Secondary => button_colors.main.secondary.dark,
                    };
                    let props = Props::new(ContentBoxItemLayout {
                        depth: Scalar::NEG_INFINITY,
                        ..Default::default()
                    }).with(ImageBoxProps {
                        material: ImageBoxMaterial::Color(ImageBoxColor {
                            color,
                            scaling: ImageBoxImageScaling::Frame(frame),
                        }),
                        ..Default::default()
                    });
                    let frame = widget! {
                        (#{"frame"} image_box: {props})
                    };
                    vec![background, frame, content]
                } else {
                    vec![background, content]
                }
            },
            ThemeVariant::Outline => {
                if let Some(frame) = paper_props.frame {
                    let button_colors = shared_props.map_or_default::<ThemeProps, _, _>(|props| {
                        props.active_colors.clone()
                    });
                    let color = match themed_props.color {
                        ThemeColor::Default => button_colors.main.default.dark,
                        ThemeColor::Primary => button_colors.main.primary.dark,
                        ThemeColor::Secondary => button_colors.main.secondary.dark,
                    };
                    let props = Props::new(ContentBoxItemLayout {
                        depth: Scalar::NEG_INFINITY,
                        ..Default::default()
                    }).with(ImageBoxProps {
                        material: ImageBoxMaterial::Color(ImageBoxColor {
                            color,
                            scaling: ImageBoxImageScaling::Frame(frame),
                        }),
                        ..Default::default()
                    });
                    let frame = widget! {
                        (#{"frame"} image_box: {props})
                    };
                    vec![frame, content]
                } else {
                    vec![content]
                }
            },
        };

        widget! {
            (#{key} content_box: {props.clone()} |[ items ]|)
        }
    }
}

widget_component! {
    pub button_paper(key, props, named_slots) {
        unpack_named_slots!(named_slots => content);
        widget! {
            (#{key} button: {props.clone()} {
                content = (#{"content"} button_paper_content: {props.clone()} {
                    content = {content}
                })
            })
        }
    }
}
