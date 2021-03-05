use crate::ui::components::app::{AppMessage, AppSharedProps};
use raui_core::prelude::*;
use raui_material::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PopupProps {
    #[serde(default)]
    pub index: usize,
    #[serde(default)]
    pub text: String,
}
implement_props_data!(PopupProps);

widget_hook! {
    use_popup(life_cycle) {
        life_cycle.change(|context| {
            for msg in context.messenger.messages {
                if let Some(msg) = msg.as_any().downcast_ref::<ButtonNotifyMessage>() {
                    if msg.trigger_start() {
                        let id = context
                            .shared_props
                            .read_cloned_or_default::<AppSharedProps>()
                            .0;
                        context.messenger.write(id, AppMessage::ClosePopup);
                    }
                }
            }
        });
    }
}

widget_component! {
    pub popup(id, key, props) [use_popup] {
        let PopupProps { index, text } = props.read_cloned_or_default::<PopupProps>();
        let button_props = Props::new(NavItemActive).with(ButtonNotifyProps(id.to_owned().into()));
        let panel_props = props.clone().with(PaperProps {
            frame: None,
            ..Default::default()
        })
        .with(VerticalBoxProps {
            separation: 10.0,
            ..Default::default()
        });
        let image_props = Props::new(ImageBoxProps {
            width: ImageBoxSizeValue::Exact(48.0),
            height: ImageBoxSizeValue::Exact(48.0),
            material: ImageBoxMaterial::Image(ImageBoxImage {
                id: format!("icon-{}", index),
                ..Default::default()
            }),
            ..Default::default()
        })
        .with(FlexBoxItemLayout {
            grow: 0.0,
            shrink: 0.0,
            fill: 1.0,
            align: 0.5,
            ..Default::default()
        });
        let text_props = TextPaperProps {
            text,
            width: TextBoxSizeValue::Fill,
            height: TextBoxSizeValue::Fill,
            use_main_color: true,
            ..Default::default()
        };

        widget! {
            (#{key} button: {button_props} {
                content = (#{"list"} vertical_paper: {panel_props} [
                    (#{"image"} image_box: {image_props})
                    (#{"text"} text_paper: {text_props})
                ])
            })
        }
    }
}
