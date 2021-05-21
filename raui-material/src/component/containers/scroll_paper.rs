use crate::{
    component::containers::paper::paper,
    theme::{ThemeColor, ThemeProps, ThemedImageMaterial, ThemedWidgetProps},
};
use raui_core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Clone, Serialize, Deserialize)]
#[props_data(raui_core::props::PropsData)]
#[prefab(raui_core::Prefab)]
pub struct SideScrollbarsPaperProps {
    #[serde(default)]
    pub size: Scalar,
    #[serde(default)]
    pub back_variant: Option<String>,
    #[serde(default)]
    pub front_variant: String,
}

impl Default for SideScrollbarsPaperProps {
    fn default() -> Self {
        Self {
            size: 10.0,
            back_variant: None,
            front_variant: Default::default(),
        }
    }
}

pub fn scroll_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        idref,
        key,
        props,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => {content, scrollbars});

    let inner_props = props.clone().without::<ContentBoxItemLayout>();

    widget! {
        (#{key} | {idref.cloned()} paper: {props.clone()} [
            (#{"scroll"} nav_scroll_box: {inner_props} {
                content = {content}
                scrollbars = {scrollbars}
            })
        ])
    }
}

pub fn scroll_paper_side_scrollbars(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        idref,
        key,
        props,
        shared_props,
        ..
    } = context;

    let scrollbars_props = props.read_cloned_or_default::<SideScrollbarsPaperProps>();
    let themed_props = props.read_cloned_or_default::<ThemedWidgetProps>();
    let colors =
        shared_props.map_or_default::<ThemeProps, _, _>(|props| props.active_colors.clone());

    let back_material = if let Some(back_variant) = &scrollbars_props.back_variant {
        let background = shared_props.map_or_default::<ThemeProps, _, _>(|props| {
            props
                .button_backgrounds
                .get(back_variant)
                .cloned()
                .unwrap_or_default()
                .default
        });
        Some(match background {
            ThemedImageMaterial::Color => {
                let color = match themed_props.color {
                    ThemeColor::Default => colors.main.default.main,
                    ThemeColor::Primary => colors.main.primary.main,
                    ThemeColor::Secondary => colors.main.secondary.main,
                };
                ImageBoxMaterial::Color(ImageBoxColor {
                    color,
                    ..Default::default()
                })
            }
            ThemedImageMaterial::Image(material) => ImageBoxMaterial::Image(material),
            ThemedImageMaterial::Procedural(material) => ImageBoxMaterial::Procedural(material),
        })
    } else {
        None
    };

    let front_material = {
        let background = shared_props.map_or_default::<ThemeProps, _, _>(|props| {
            props
                .button_backgrounds
                .get(&scrollbars_props.front_variant)
                .cloned()
                .unwrap_or_default()
                .trigger
        });
        match background {
            ThemedImageMaterial::Color => {
                let color = match themed_props.color {
                    ThemeColor::Default => colors.main.default.main,
                    ThemeColor::Primary => colors.main.primary.main,
                    ThemeColor::Secondary => colors.main.secondary.main,
                };
                ImageBoxMaterial::Color(ImageBoxColor {
                    color,
                    ..Default::default()
                })
            }
            ThemedImageMaterial::Image(material) => ImageBoxMaterial::Image(material),
            ThemedImageMaterial::Procedural(material) => ImageBoxMaterial::Procedural(material),
        }
    };

    props.write(SideScrollbarsProps {
        size: scrollbars_props.size,
        back_material,
        front_material,
    });

    widget! {
        (#{key} | {idref.cloned()} nav_scroll_box_side_scrollbars: {props.clone()})
    }
}
