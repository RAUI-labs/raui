use crate::theme::ThemeProps;
use raui_core::{
    PropsData, make_widget, unpack_named_slots,
    widget::{
        component::{
            containers::{content_box::content_box, portal_box::portal_box},
            image_box::{ImageBoxProps, image_box},
            interactive::navigation::navigation_barrier,
        },
        context::WidgetContext,
        node::WidgetNode,
        unit::image::{ImageBoxColor, ImageBoxMaterial},
        utils::Color,
    },
};
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Clone, Serialize, Deserialize)]
#[props_data(raui_core::props::PropsData)]
#[prefab(raui_core::Prefab)]
pub struct ModalPaperProps {
    #[serde(default = "ModalPaperProps::default_shadow_shown")]
    pub shadow_shown: bool,
    #[serde(default)]
    pub shadow_variant: String,
}

impl ModalPaperProps {
    fn default_shadow_shown() -> bool {
        true
    }
}

impl Default for ModalPaperProps {
    fn default() -> Self {
        Self {
            shadow_shown: Self::default_shadow_shown(),
            shadow_variant: Default::default(),
        }
    }
}

pub fn modal_paper(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        props,
        shared_props,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => content);

    let ModalPaperProps {
        shadow_shown,
        shadow_variant,
    } = props.read_cloned_or_default();

    let mut color = Color::transparent();
    if shadow_shown {
        if let Ok(props) = shared_props.read::<ThemeProps>() {
            if let Some(c) = props.modal_shadow_variants.get(&shadow_variant) {
                color = *c;
            }
        }
    }

    let shadow_image_props = ImageBoxProps {
        material: ImageBoxMaterial::Color(ImageBoxColor {
            color,
            ..Default::default()
        }),
        ..Default::default()
    };

    make_widget!(portal_box)
        .key(key)
        .named_slot(
            "content",
            make_widget!(content_box)
                .key("container")
                .listed_slot(
                    make_widget!(navigation_barrier)
                        .key("shadow-barrier")
                        .named_slot(
                            "content",
                            make_widget!(image_box)
                                .key("shadow-image")
                                .with_props(shadow_image_props),
                        ),
                )
                .listed_slot(content),
        )
        .into()
}
