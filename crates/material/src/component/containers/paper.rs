use crate::theme::{ThemeColor, ThemeProps, ThemeVariant, ThemedImageMaterial, ThemedWidgetProps};
use raui_core::{
    PropsData, Scalar, make_widget,
    props::Props,
    widget::{
        component::{
            WidgetComponent,
            containers::content_box::{content_box, nav_content_box},
            image_box::{ImageBoxProps, image_box},
        },
        context::WidgetContext,
        node::WidgetNode,
        unit::{
            content::ContentBoxItemLayout,
            image::{ImageBoxColor, ImageBoxFrame, ImageBoxImageScaling, ImageBoxMaterial},
        },
    },
};
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(raui_core::props::PropsData)]
#[prefab(raui_core::Prefab)]
pub struct PaperProps {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame: Option<ImageBoxFrame>,
    #[serde(default)]
    pub variant: String,
}

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(raui_core::props::PropsData)]
#[prefab(raui_core::Prefab)]
pub struct PaperContentLayoutProps(pub ContentBoxItemLayout);

pub fn nav_paper(context: WidgetContext) -> WidgetNode {
    paper_impl(make_widget!(nav_content_box), context)
}

pub fn paper(context: WidgetContext) -> WidgetNode {
    paper_impl(make_widget!(content_box), context)
}

pub fn paper_impl(component: WidgetComponent, context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        idref,
        key,
        props,
        shared_props,
        listed_slots,
        ..
    } = context;

    let paper_props = props.read_cloned_or_default::<PaperProps>();
    let themed_props = props.read_cloned_or_default::<ThemedWidgetProps>();
    let listed_slots = listed_slots
        .into_iter()
        .map(|mut item| {
            item.remap_props(|mut props| {
                if let Ok(PaperContentLayoutProps(layout)) = props.consume_unwrap_cloned() {
                    props.write(layout);
                }
                props
            });
            item
        })
        .collect::<Vec<_>>();

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
            let background = make_widget!(image_box)
                .key("background")
                .merge_props(props)
                .into();
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
                let frame = make_widget!(image_box)
                    .key("frame")
                    .merge_props(props)
                    .into();
                std::iter::once(background)
                    .chain(std::iter::once(frame))
                    .chain(listed_slots)
                    .collect::<Vec<_>>()
            } else {
                std::iter::once(background)
                    .chain(listed_slots)
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
                let frame = make_widget!(image_box)
                    .key("frame")
                    .merge_props(props)
                    .into();
                std::iter::once(frame)
                    .chain(listed_slots)
                    .collect::<Vec<_>>()
            } else {
                listed_slots
            }
        }
    };

    component
        .key(key)
        .maybe_idref(idref.cloned())
        .merge_props(props.clone())
        .listed_slots(items)
        .into()
}
