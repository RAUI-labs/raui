use crate::theme::ThemeProps;
use raui_core::prelude::*;
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

#[derive(PropsData, Default, Debug, Clone, Serialize, Deserialize)]
#[props_data(raui_core::props::PropsData)]
#[prefab(raui_core::Prefab)]
pub struct ModalsContainer(#[serde(default)] pub WidgetRef);

impl Default for ModalPaperProps {
    fn default() -> Self {
        Self {
            shadow_shown: Self::default_shadow_shown(),
            shadow_variant: Default::default(),
        }
    }
}

impl ModalPaperProps {
    fn default_shadow_shown() -> bool {
        true
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
    let ModalsContainer(modals_idref) = shared_props.read_cloned_or_default();

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

    widget! {
        (#{key} portal_box: {PortalBoxProps(modals_idref)} {
            content = (#{"container"} content_box [
                (#{"shadow-barrier"} navigation_barrier {
                    content = (#{"shadow-image"} image_box: {shadow_image_props})
                })
                {content}
            ])
        })
    }
}
