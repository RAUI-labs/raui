use crate::{
    component::containers::paper::PaperProps,
    theme::{ThemeColor, ThemeProps, ThemeVariant, ThemedImageMaterial, ThemedWidgetProps},
};
use raui_core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[props_data(raui_core::props::PropsData)]
#[prefab(raui_core::Prefab)]
pub enum ButtonPaperOverrideStyle {
    #[default]
    None,
    Default,
    Selected,
    Triggered,
}

fn button_paper_content(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        props,
        shared_props,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => content);

    let mut button_props = props.read_cloned_or_default::<ButtonProps>();
    let paper_props = props.read_cloned_or_default::<PaperProps>();
    let themed_props = props.read_cloned_or_default::<ThemedWidgetProps>();
    let override_style = props.read_cloned_or_default::<ButtonPaperOverrideStyle>();

    if override_style != ButtonPaperOverrideStyle::None {
        button_props.selected = override_style == ButtonPaperOverrideStyle::Selected;
        button_props.trigger = override_style == ButtonPaperOverrideStyle::Triggered;
        button_props.context = false;
    }

    let items = match themed_props.variant {
        ThemeVariant::ContentOnly => vec![content],
        ThemeVariant::Filled => {
            let button_background = shared_props.map_or_default::<ThemeProps, _, _>(|props| {
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
            let button_colors = shared_props
                .map_or_default::<ThemeProps, _, _>(|props| props.active_colors.clone());
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
            let background = make_widget!(image_box)
                .key("background")
                .merge_props(props)
                .into();
            if let Some(frame) = paper_props.frame {
                let color = match themed_props.color {
                    ThemeColor::Default => button_colors.main.default.dark,
                    ThemeColor::Primary => button_colors.main.primary.dark,
                    ThemeColor::Secondary => button_colors.main.secondary.dark,
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
                let frame = make_widget!(image_box)
                    .key("frame")
                    .merge_props(props)
                    .into();
                vec![background, frame, content]
            } else {
                vec![background, content]
            }
        }
        ThemeVariant::Outline => {
            if let Some(frame) = paper_props.frame {
                let button_colors = shared_props
                    .map_or_default::<ThemeProps, _, _>(|props| props.active_colors.clone());
                let color = match themed_props.color {
                    ThemeColor::Default => button_colors.main.default.dark,
                    ThemeColor::Primary => button_colors.main.primary.dark,
                    ThemeColor::Secondary => button_colors.main.secondary.dark,
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
                let frame = make_widget!(image_box)
                    .key("frame")
                    .merge_props(props)
                    .into();
                vec![frame, content]
            } else {
                vec![content]
            }
        }
    };

    make_widget!(content_box)
        .key(key)
        .merge_props(props.clone())
        .listed_slots(items)
        .into()
}

pub fn button_paper(context: WidgetContext) -> WidgetNode {
    button_paper_impl(make_widget!(button), context)
}

pub fn button_paper_impl(component: WidgetComponent, context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        idref,
        key,
        props,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => content);

    component
        .key(key)
        .maybe_idref(idref.cloned())
        .merge_props(props.clone())
        .named_slot(
            "content",
            make_widget!(button_paper_content)
                .key("content")
                .merge_props(props.clone())
                .named_slot("content", content),
        )
        .into()
}
