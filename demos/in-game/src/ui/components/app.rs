use crate::ui::components::{
    inventory::inventory,
    item_cell::{ItemCellProps, ItemCellsProps},
    minimap::minimap,
    new_theme,
    popup::{popup, PopupProps},
};
use raui_core::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Clone, Serialize, Deserialize)]
pub struct AppProps {
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub texts: Vec<String>,
}

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
pub struct AppSharedProps(pub WidgetId);

#[derive(PropsData, Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub popup_index: Option<usize>,
}

#[derive(MessageData, Debug, Clone)]
pub enum AppMessage {
    ShowPopup(usize),
    ClosePopup,
}

fn use_app(context: &mut WidgetContext) {
    context.life_cycle.change(|context| {
        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref() {
                match msg {
                    AppMessage::ShowPopup(index) => {
                        let _ = context.state.write(AppState {
                            popup_index: Some(*index),
                        });
                    }
                    AppMessage::ClosePopup => {
                        let _ = context.state.write(AppState { popup_index: None });
                    }
                }
            }
        }
    });
}

#[pre_hooks(use_nav_container_active, use_app)]
pub fn app(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        key,
        props,
        state,
        ..
    } = context;

    let shared_props = Props::new(AppSharedProps(id.to_owned())).with(new_theme());
    let minimap_props = ContentBoxItemLayout {
        anchors: Rect {
            left: 1.0,
            right: 1.0,
            top: 0.0,
            bottom: 0.0,
        },
        align: Vec2 { x: 1.0, y: 0.0 },
        offset: Vec2 { x: -6.0, y: 6.0 },
        ..Default::default()
    };
    let inventory_props = Props::new(ContentBoxItemLayout {
        anchors: Rect {
            left: 0.5,
            right: 0.5,
            top: 1.0,
            bottom: 1.0,
        },
        align: Vec2 { x: 0.5, y: 1.0 },
        offset: Vec2 { x: 0.0, y: -6.0 },
        ..Default::default()
    })
    .with(ItemCellsProps {
        items: (0..=18)
            .map(|i| ItemCellProps {
                image: format!("icon-{}", i),
                thin: false,
            })
            .collect::<Vec<_>>(),
    });
    let popup = match state.read::<AppState>() {
        Ok(data) => {
            if let Some(index) = data.popup_index {
                let text = match props.read::<AppProps>() {
                    Ok(props) => props.texts.get(index).cloned().unwrap_or_default(),
                    Err(_) => String::new(),
                };
                let popup_props = Props::new(ContentBoxItemLayout {
                    margin: Rect {
                        left: 20.0,
                        right: 20.0,
                        top: 20.0,
                        bottom: 46.0,
                    },
                    ..Default::default()
                })
                .with(PopupProps { index, text });

                widget! {
                    (#{"popup"} popup: {popup_props})
                }
            } else {
                widget! {()}
            }
        }
        Err(_) => {
            widget! {()}
        }
    };

    widget! {(#{key} content_box: {props.clone()} | {shared_props} [
        (#{"minimap"} minimap: {minimap_props})
        (#{"inventory"} inventory: {inventory_props})
        {popup}
    ])}
}
