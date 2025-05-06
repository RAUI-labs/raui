use crate::{
    component::text_paper::{TextPaperProps, text_paper},
    theme::{ThemeColor, ThemeProps, ThemedImageMaterial, ThemedSliderMaterial},
};
use raui_core::{
    PropsData, make_widget, unpack_named_slots,
    widget::{
        component::{
            WidgetComponent,
            containers::content_box::content_box,
            image_box::{ImageBoxProps, image_box},
            interactive::slider_view::{SliderViewDirection, SliderViewProps, slider_view},
        },
        context::WidgetContext,
        node::WidgetNode,
        unit::{
            content::ContentBoxItemLayout,
            image::{ImageBoxColor, ImageBoxMaterial},
            text::TextBoxSizeValue,
        },
        utils::Rect,
    },
};
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Clone, Serialize, Deserialize)]
#[props_data(raui_core::props::PropsData)]
#[prefab(raui_core::Prefab)]
pub struct SliderPaperProps {
    #[serde(default)]
    pub variant: String,
    #[serde(default = "SliderPaperProps::default_background_color")]
    pub background_color: ThemeColor,
    #[serde(default = "SliderPaperProps::default_filling_color")]
    pub filling_color: ThemeColor,
}

impl Default for SliderPaperProps {
    fn default() -> Self {
        Self {
            variant: Default::default(),
            background_color: Self::default_background_color(),
            filling_color: Self::default_filling_color(),
        }
    }
}

impl SliderPaperProps {
    fn default_background_color() -> ThemeColor {
        ThemeColor::Secondary
    }

    fn default_filling_color() -> ThemeColor {
        ThemeColor::Primary
    }
}

#[derive(PropsData, Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[props_data(raui_core::props::PropsData)]
#[prefab(raui_core::Prefab)]
pub struct NumericSliderPaperProps {
    #[serde(default)]
    pub fractional_digits_count: Option<usize>,
}

pub fn slider_paper(context: WidgetContext) -> WidgetNode {
    slider_paper_impl(make_widget!(slider_view), context)
}

pub fn slider_paper_impl(component: WidgetComponent, context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        idref,
        key,
        props,
        shared_props,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => content);

    let SliderPaperProps {
        variant,
        background_color,
        filling_color,
    } = props.read_cloned_or_default();
    let anchors = props
        .read::<SliderViewProps>()
        .ok()
        .map(|props| {
            let percentage = props.get_percentage();
            match props.direction {
                SliderViewDirection::LeftToRight => Rect {
                    left: 0.0,
                    right: percentage,
                    top: 0.0,
                    bottom: 1.0,
                },
                SliderViewDirection::RightToLeft => Rect {
                    left: 1.0 - percentage,
                    right: 1.0,
                    top: 0.0,
                    bottom: 1.0,
                },
                SliderViewDirection::TopToBottom => Rect {
                    left: 0.0,
                    right: 1.0,
                    top: 0.0,
                    bottom: percentage,
                },
                SliderViewDirection::BottomToTop => Rect {
                    left: 0.0,
                    right: 1.0,
                    top: 1.0 - percentage,
                    bottom: 1.0,
                },
            }
        })
        .unwrap_or_default();
    let (background, filling) = match shared_props.read::<ThemeProps>() {
        Ok(props) => {
            if let Some(material) = props.slider_variants.get(&variant).cloned() {
                let background_color = match background_color {
                    ThemeColor::Default => props.active_colors.main.default.main,
                    ThemeColor::Primary => props.active_colors.main.primary.main,
                    ThemeColor::Secondary => props.active_colors.main.secondary.main,
                };
                let filling_color = match filling_color {
                    ThemeColor::Default => props.active_colors.main.default.main,
                    ThemeColor::Primary => props.active_colors.main.primary.main,
                    ThemeColor::Secondary => props.active_colors.main.secondary.main,
                };
                let ThemedSliderMaterial {
                    background,
                    filling,
                } = material;
                let background = match background {
                    ThemedImageMaterial::Color => ImageBoxProps {
                        material: ImageBoxMaterial::Color(ImageBoxColor {
                            color: background_color,
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                    ThemedImageMaterial::Image(mut data) => {
                        data.tint = filling_color;
                        ImageBoxProps {
                            material: ImageBoxMaterial::Image(data),
                            ..Default::default()
                        }
                    }
                    ThemedImageMaterial::Procedural(data) => ImageBoxProps {
                        material: ImageBoxMaterial::Procedural(data),
                        ..Default::default()
                    },
                };
                let filling = match filling {
                    ThemedImageMaterial::Color => ImageBoxProps {
                        material: ImageBoxMaterial::Color(ImageBoxColor {
                            color: filling_color,
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                    ThemedImageMaterial::Image(mut data) => {
                        data.tint = filling_color;
                        ImageBoxProps {
                            material: ImageBoxMaterial::Image(data),
                            ..Default::default()
                        }
                    }
                    ThemedImageMaterial::Procedural(data) => ImageBoxProps {
                        material: ImageBoxMaterial::Procedural(data),
                        ..Default::default()
                    },
                };
                (background, filling)
            } else {
                Default::default()
            }
        }
        Err(_) => Default::default(),
    };

    component
        .key(key)
        .maybe_idref(idref.cloned())
        .merge_props(props.clone())
        .named_slot(
            "content",
            make_widget!(content_box)
                .key("content")
                .merge_props(props.clone())
                .listed_slot(
                    make_widget!(image_box)
                        .key("background")
                        .with_props(background),
                )
                .listed_slot(
                    make_widget!(image_box)
                        .key("filling")
                        .with_props(ContentBoxItemLayout {
                            anchors,
                            ..Default::default()
                        })
                        .with_props(filling),
                )
                .listed_slot(content),
        )
        .into()
}

pub fn numeric_slider_paper(context: WidgetContext) -> WidgetNode {
    numeric_slider_paper_impl(make_widget!(slider_paper), context)
}

pub fn numeric_slider_paper_impl(component: WidgetComponent, context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        idref, key, props, ..
    } = context;

    let mut text = props.read_cloned_or_default::<TextPaperProps>();
    text.width = TextBoxSizeValue::Fill;
    text.height = TextBoxSizeValue::Fill;
    let value = props
        .read::<SliderViewProps>()
        .ok()
        .map(|props| props.get_value())
        .unwrap_or_default();
    text.text = if let Some(count) = props
        .read_cloned_or_default::<NumericSliderPaperProps>()
        .fractional_digits_count
    {
        format!("{:.1$}", value, count)
    } else {
        value.to_string()
    };

    component
        .key(key)
        .maybe_idref(idref.cloned())
        .merge_props(props.clone())
        .named_slot(
            "content",
            make_widget!(text_paper)
                .merge_props(props.clone())
                .with_props(text),
        )
        .into()
}
