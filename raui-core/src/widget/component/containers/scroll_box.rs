use crate::{
    props::Props,
    unpack_named_slots, widget,
    widget::{
        component::{
            containers::content_box::content_box,
            image_box::{image_box, ImageBoxProps},
            interactive::{
                button::{button, ButtonNotifyMessage, ButtonNotifyProps},
                navigation::{
                    use_nav_container_active, use_nav_item, use_nav_item_active,
                    use_nav_scroll_view_content, NavButtonTrackingActive, NavItemActive, NavJump,
                    NavScroll, NavSignal,
                },
                scroll_view::{use_scroll_view, ScrollViewState},
            },
            use_resize_listener, ResizeListenerSignal,
        },
        unit::{area::AreaBoxNode, content::ContentBoxItemLayout, image::ImageBoxMaterial},
        utils::{Rect, Vec2},
        WidgetId,
    },
    widget_component, widget_hook, Scalar,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ScrollBoxOwner(
    #[serde(default)]
    #[serde(skip_serializing_if = "WidgetId::is_none")]
    pub WidgetId,
);
implement_props_data!(ScrollBoxOwner);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideScrollbarsProps {
    #[serde(default)]
    pub size: Scalar,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub back_material: Option<ImageBoxMaterial>,
    #[serde(default)]
    pub front_material: ImageBoxMaterial,
}
implement_props_data!(SideScrollbarsProps);

impl Default for SideScrollbarsProps {
    fn default() -> Self {
        Self {
            size: 10.0,
            back_material: None,
            front_material: Default::default(),
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct SideScrollbarsState {
    pub horizontal: Scalar,
    pub vertical: Scalar,
}
implement_props_data!(SideScrollbarsState);

widget_hook! {
    use_nav_scroll_box_content(life_cycle) {
        life_cycle.change(|context| {
            for msg in context.messenger.messages {
                if let Some(ResizeListenerSignal::Change) = msg.as_any().downcast_ref() {
                    if let Ok(data) = context.props.read::<ScrollBoxOwner>() {
                        context.messenger.write(data.0.to_owned(), ResizeListenerSignal::Change);
                    }
                }
            }
        });
    }
}

widget_component! {
    nav_scroll_box_content(id, named_slots) [
        use_resize_listener,
        use_nav_item_active,
        use_nav_container_active,
        use_nav_scroll_view_content,
        use_nav_scroll_box_content,
    ] {
        unpack_named_slots!(named_slots => content);

        widget! {{{
            AreaBoxNode {
                id: id.to_owned(),
                slot: Box::new(content),
            }
        }}}
    }
}

widget_hook! {
    use_nav_scroll_box(life_cycle) {
        life_cycle.change(|context| {
            for msg in context.messenger.messages {
                if let Some(ResizeListenerSignal::Change) = msg.as_any().downcast_ref() {
                    if let Ok(data) = context.state.read::<ScrollViewState>() {
                        context.signals.write(NavSignal::Jump(
                            NavJump::Scroll(NavScroll::Factor(data.value, false)),
                        ));
                    }
                }
            }
        });
    }
}

widget_component! {
    pub nav_scroll_box(id, key, props, state, named_slots) [
        use_resize_listener,
        use_nav_item,
        use_nav_container_active,
        use_scroll_view,
        use_nav_scroll_box,
    ] {
        unpack_named_slots!(named_slots => {content, scrollbars});

        let scroll_props = state.read_cloned_or_default::<ScrollViewState>();
        let content_props = Props::new(ContentBoxItemLayout {
            align: scroll_props.value,
            ..Default::default()
        })
        .with(ScrollBoxOwner(id.to_owned()));
        if let Some(props) = scrollbars.props_mut() {
            props.write(ScrollBoxOwner(id.to_owned()));
            props.write(scroll_props);
        }

        widget! {
            (#{key} content_box: {props.clone()} [
                (#{"content"} nav_scroll_box_content: {content_props} {
                    content = {content}
                })
                {scrollbars}
            ])
        }
    }
}

widget_hook! {
    use_nav_scroll_box_side_scrollbars(life_cycle) {
        life_cycle.mount(|context| {
            drop(context.state.write_with(SideScrollbarsState::default()));
        });

        life_cycle.change(|context| {
            for msg in context.messenger.messages {
                if let Some(msg) = msg.as_any().downcast_ref::<ButtonNotifyMessage>() {
                    if msg.state.selected && (msg.state.trigger || msg.state.context) {
                        let mut dirty = false;
                        let mut data = context.state.read_cloned_or_default::<SideScrollbarsState>();
                        if msg.sender.key() == "hbar" {
                            data.horizontal = msg.state.pointer.x;
                            dirty = true;
                        } else if msg.sender.key() == "vbar" {
                            data.vertical = msg.state.pointer.y;
                            dirty = true;
                        }
                        if dirty {
                            let view = context.props
                                .read_cloned_or_default::<ScrollBoxOwner>().0;
                            let pos = Vec2 {
                                x: data.horizontal,
                                y: data.vertical,
                            };
                            context.signals.write(NavSignal::Jump(
                                NavJump::Scroll(NavScroll::DirectFactor(view.into(), pos, false)),
                            ));
                            drop(context.state.write_with(data));
                        }
                    }
                }
            }
        });
    }
}

widget_component! {
    pub nav_scroll_box_side_scrollbars(id, key, props) [
        use_nav_item_active,
        use_nav_container_active,
        use_nav_scroll_box_side_scrollbars,
    ] {
        let view_props = props.read_cloned_or_default::<ScrollViewState>();
        let SideScrollbarsProps {
            size,
            back_material,
            front_material,
        } = props.read_cloned_or_default();
        let hbar = if view_props.size_factor.x > 1.0 {
            let props = Props::new(ButtonNotifyProps(id.to_owned().into()))
            .with(NavItemActive)
            .with(NavButtonTrackingActive)
            .with(ContentBoxItemLayout {
                anchors: Rect {
                    left: 0.0,
                    right: 1.0,
                    top: 1.0,
                    bottom: 1.0,
                },
                margin: Rect {
                    left: 0.0,
                    right: size,
                    top: -size,
                    bottom: 0.0,
                },
                align: Vec2 { x: 0.0, y: 1.0 },
                ..Default::default()
            });
            let front_props = ImageBoxProps {
                material: front_material.clone(),
                ..Default::default()
            };
            let back = if let Some(material) = back_material.clone() {
                let props = ImageBoxProps {
                    material,
                    ..Default::default()
                };

                widget!{ (#{"back"} image_box: {props}) }
            } else {
                widget!{()}
            };

            widget! {
                (#{"hbar"} button: {props} {
                    content = (#{"container"} content_box [
                        {back}
                        (#{"front"} image_box: {front_props})
                    ])
                })
            }
        } else {
            widget!{()}
        };
        let vbar = if view_props.size_factor.y > 1.0 {
            let props = Props::new(ButtonNotifyProps(id.to_owned().into()))
            .with(NavItemActive)
            .with(NavButtonTrackingActive)
            .with(ContentBoxItemLayout {
                anchors: Rect {
                    left: 1.0,
                    right: 1.0,
                    top: 0.0,
                    bottom: 1.0,
                },
                margin: Rect {
                    left: -size,
                    right: 0.0,
                    top: 0.0,
                    bottom: size,
                },
                align: Vec2 { x: 1.0, y: 0.0 },
                ..Default::default()
            });
            let back = if let Some(material) = back_material {
                let props = ImageBoxProps {
                    material,
                    ..Default::default()
                };

                widget!{ (#{"back"} image_box: {props}) }
            } else {
                widget!{()}
            };
            let front_props = ImageBoxProps {
                material: front_material,
                ..Default::default()
            };

            widget! {
                (#{"vbar"} button: {props} {
                    content = (#{"container"} content_box [
                        {back}
                        (#{"front"} image_box: {front_props})
                    ])
                })
            }
        } else {
            widget!{()}
        };

        widget! {
            (#{key} content_box [
                {hbar}
                {vbar}
            ])
        }
    }
}
