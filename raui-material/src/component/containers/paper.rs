use crate::theme::{ThemeColor, ThemeProps, ThemeVariant, ThemedImageMaterial, ThemedWidgetProps};
use raui_core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Clone, Serialize, Deserialize)]
#[props_data(raui_core::props::PropsData)]
#[prefab(raui_core::Prefab)]
pub struct PaperProps {
    #[serde(default = "PaperProps::default_frame")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame: Option<ImageBoxFrame>,
    #[serde(default)]
    pub variant: String,
}

impl PaperProps {
    #[allow(clippy::unnecessary_wraps)]
    fn default_frame() -> Option<ImageBoxFrame> {
        Some(2.0.into())
    }
}

impl Default for PaperProps {
    fn default() -> Self {
        Self {
            frame: Self::default_frame(),
            variant: Default::default(),
        }
    }
}

pub fn paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        props,
        shared_props,
        listed_slots,
        ..
    } = context;

    let paper_props = props.read_cloned_or_default::<PaperProps>();
    let themed_props = props.read_cloned_or_default::<ThemedWidgetProps>();

    let items = match themed_props.variant {
        ThemeVariant::ContentOnly => listed_slots,
        ThemeVariant::Filled => {
            let content_background = shared_props.map_or_default::<ThemeProps, _, _>(|props| {
                props
                    .content_backgrounds
                    .get(&paper_props.variant)
                    .cloned()
                    .unwrap_or_default()
            });
            let background_colors = shared_props
                .map_or_default::<ThemeProps, _, _>(|props| props.background_colors.clone());
            let image = match content_background {
                ThemedImageMaterial::Color => {
                    let color = match themed_props.color {
                        ThemeColor::Default => background_colors.main.default.main,
                        ThemeColor::Primary => background_colors.main.primary.main,
                        ThemeColor::Secondary => background_colors.main.secondary.main,
                    };
                    ImageBoxProps {
                        material: ImageBoxMaterial::Color(ImageBoxColor {
                            color,
                            ..Default::default()
                        }),
                        ..Default::default()
                    }
                }
                ThemedImageMaterial::Image(material) => ImageBoxProps {
                    material: ImageBoxMaterial::Image(material),
                    ..Default::default()
                },
                ThemedImageMaterial::Procedural(material) => ImageBoxProps {
                    material: ImageBoxMaterial::Procedural(material),
                    ..Default::default()
                },
            };
            let props = Props::new(ContentBoxItemLayout {
                depth: Scalar::NEG_INFINITY,
                ..Default::default()
            })
            .with(image);
            let background = widget! {
                (#{"background"} image_box: {props})
            };
            if let Some(frame) = paper_props.frame {
                let color = match themed_props.color {
                    ThemeColor::Default => background_colors.main.default.dark,
                    ThemeColor::Primary => background_colors.main.primary.dark,
                    ThemeColor::Secondary => background_colors.main.secondary.dark,
                };
                let props = Props::new(ContentBoxItemLayout {
                    depth: Scalar::NEG_INFINITY,
                    ..Default::default()
                })
                .with(ImageBoxProps {
                    material: ImageBoxMaterial::Color(ImageBoxColor {
                        color,
                        scaling: ImageBoxImageScaling::Frame(frame),
                    }),
                    ..Default::default()
                });
                let frame = widget! {
                    (#{"frame"} image_box: {props})
                };
                std::iter::once(background)
                    .chain(std::iter::once(frame))
                    .chain(listed_slots.into_iter())
                    .collect::<Vec<_>>()
            } else {
                std::iter::once(background)
                    .chain(listed_slots.into_iter())
                    .collect::<Vec<_>>()
            }
        }
        ThemeVariant::Outline => {
            if let Some(frame) = paper_props.frame {
                let background_colors = shared_props
                    .map_or_default::<ThemeProps, _, _>(|props| props.background_colors.clone());
                let color = match themed_props.color {
                    ThemeColor::Default => background_colors.main.default.dark,
                    ThemeColor::Primary => background_colors.main.primary.dark,
                    ThemeColor::Secondary => background_colors.main.secondary.dark,
                };
                let props = Props::new(ContentBoxItemLayout {
                    depth: Scalar::NEG_INFINITY,
                    ..Default::default()
                })
                .with(ImageBoxProps {
                    material: ImageBoxMaterial::Color(ImageBoxColor {
                        color,
                        scaling: ImageBoxImageScaling::Frame(frame),
                    }),
                    ..Default::default()
                });
                let frame = widget! {
                    (#{"frame"} image_box: {props})
                };
                std::iter::once(frame)
                    .chain(listed_slots.into_iter())
                    .collect::<Vec<_>>()
            } else {
                listed_slots
            }
        }
    };

    widget! {
        (#{key} content_box: {props.clone()} |[ items ]|)
    }
}
