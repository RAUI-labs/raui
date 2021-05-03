use crate::ui::components::{
    app::{AppMessage, AppSharedProps},
    inventory::InventoryMessage,
};
use raui_core::prelude::*;
use raui_material::prelude::*;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
pub struct ItemCellProps {
    #[serde(default)]
    pub image: String,
    #[serde(default)]
    pub thin: bool,
}

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
pub struct ItemData {
    pub index: usize,
}

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
pub struct ItemCellsProps {
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<ItemCellProps>,
}

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
pub struct OwningInventoryProps(pub WidgetId);

fn use_item_cell(context: &mut WidgetContext) {
    context.life_cycle.change(|context| {
        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<ButtonNotifyMessage>() {
                if msg.trigger_start() {
                    let _ = context.animator.change(
                        "",
                        Some(Animation::Value(AnimatedValue {
                            name: "click".to_owned(),
                            duration: 0.15,
                        })),
                    );
                    match msg.sender.key() {
                        "prev" => {
                            let id = context
                                .shared_props
                                .read_cloned_or_default::<OwningInventoryProps>()
                                .0;
                            context.messenger.write(id, InventoryMessage::Prev);
                        }
                        "next" => {
                            let id = context
                                .shared_props
                                .read_cloned_or_default::<OwningInventoryProps>()
                                .0;
                            context.messenger.write(id, InventoryMessage::Next);
                        }
                        _ => {
                            if let Ok(data) = context.props.read::<ItemData>() {
                                let id = context
                                    .shared_props
                                    .read_cloned_or_default::<AppSharedProps>()
                                    .0;
                                context
                                    .messenger
                                    .write(id, AppMessage::ShowPopup(data.index));
                            }
                        }
                    }
                }
            }
        }
    });
}

#[pre_hooks(use_item_cell)]
pub fn item_cell(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        key,
        props,
        animator,
        ..
    } = context;

    let ItemCellProps { image, thin } = props.read_cloned_or_default();
    let button_props = props
        .clone()
        .with(NavItemActive)
        .with(ButtonNotifyProps(id.to_owned().into()));
    let size_props = SizeBoxProps {
        width: SizeBoxSizeValue::Exact(if thin { 18.0 } else { 24.0 }),
        height: SizeBoxSizeValue::Exact(24.0),
        margin: Rect {
            left: if thin { -4.0 } else { 1.0 },
            right: if thin { -4.0 } else { 1.0 },
            top: 2.0,
            bottom: 2.0,
        },
        ..Default::default()
    };
    let panel_props = props.clone().with(PaperProps {
        variant: "cell".to_owned(),
        frame: None,
    });
    let component = if thin { content_box } else { paper };

    if image.is_empty() {
        widget! {
            (#{key} button: {button_props} {
                content = (#{"resize"} size_box: {size_props} {
                    content = (#{"panel"} component: {panel_props})
                })
            })
        }
    } else {
        let scale = lerp(
            1.0,
            1.5,
            (animator.value_progress_or_zero("", "click") * PI).sin(),
        );
        let image_props = Props::new(ImageBoxProps {
            content_keep_aspect_ratio: Some(ImageBoxAspectRatio {
                horizontal_alignment: 0.5,
                vertical_alignment: 0.5,
            }),
            material: ImageBoxMaterial::Image(ImageBoxImage {
                id: image,
                ..Default::default()
            }),
            transform: Transform {
                pivot: Vec2 { x: 0.5, y: 0.5 },
                scale: Vec2 { x: scale, y: scale },
                ..Default::default()
            },
            ..Default::default()
        })
        .with(ContentBoxItemLayout {
            margin: Rect {
                left: 4.0,
                right: 4.0,
                top: 4.0,
                bottom: 4.0,
            },
            ..Default::default()
        });

        widget! {
            (#{key} button: {button_props} {
                content = (#{"resize"} size_box: {size_props} {
                    content = (#{"panel"} component: {panel_props} [
                        (#{"icon"} image_box: {image_props})
                    ])
                })
            })
        }
    }
}
