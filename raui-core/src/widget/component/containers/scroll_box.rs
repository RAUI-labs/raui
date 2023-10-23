use crate::{
    make_widget, pre_hooks,
    props::Props,
    unpack_named_slots,
    widget::{
        component::{
            containers::{
                content_box::{content_box, ContentBoxProps},
                size_box::{size_box, SizeBoxProps},
            },
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
        context::WidgetContext,
        node::WidgetNode,
        unit::{
            area::AreaBoxNode, content::ContentBoxItemLayout, image::ImageBoxMaterial,
            size::SizeBoxSizeValue,
        },
        utils::{lerp, Rect, Vec2},
        WidgetId,
    },
    PropsData, Scalar,
};
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct ScrollBoxOwner(
    #[serde(default)]
    #[serde(skip_serializing_if = "WidgetId::is_none")]
    pub WidgetId,
);

#[derive(PropsData, Debug, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct SideScrollbarsProps {
    #[serde(default)]
    pub size: Scalar,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub back_material: Option<ImageBoxMaterial>,
    #[serde(default)]
    pub front_material: ImageBoxMaterial,
}

impl Default for SideScrollbarsProps {
    fn default() -> Self {
        Self {
            size: 10.0,
            back_material: None,
            front_material: Default::default(),
        }
    }
}

#[derive(PropsData, Debug, Default, Copy, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct SideScrollbarsState {
    pub horizontal: Scalar,
    pub vertical: Scalar,
}

pub fn use_nav_scroll_box_content(context: &mut WidgetContext) {
    context.life_cycle.change(|context| {
        for msg in context.messenger.messages {
            if let Some(ResizeListenerSignal::Change(size)) = msg.as_any().downcast_ref() {
                if let Ok(data) = context.props.read::<ScrollBoxOwner>() {
                    context
                        .messenger
                        .write(data.0.to_owned(), ResizeListenerSignal::Change(*size));
                }
            }
        }
    });
}

#[pre_hooks(
    use_resize_listener,
    use_nav_item_active,
    use_nav_container_active,
    use_nav_scroll_view_content,
    use_nav_scroll_box_content
)]
pub fn nav_scroll_box_content(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id, named_slots, ..
    } = context;
    unpack_named_slots!(named_slots => content);

    AreaBoxNode {
        id: id.to_owned(),
        slot: Box::new(content),
    }
    .into()
}

pub fn use_nav_scroll_box(context: &mut WidgetContext) {
    context.life_cycle.change(|context| {
        for msg in context.messenger.messages {
            if let Some(ResizeListenerSignal::Change(_)) = msg.as_any().downcast_ref() {
                if let Ok(data) = context.state.read::<ScrollViewState>() {
                    context
                        .signals
                        .write(NavSignal::Jump(NavJump::Scroll(NavScroll::Factor(
                            data.value, false,
                        ))));
                }
            }
        }
    });
}

#[pre_hooks(
    use_resize_listener,
    use_nav_item,
    use_nav_container_active,
    use_scroll_view,
    use_nav_scroll_box
)]
pub fn nav_scroll_box(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        key,
        props,
        state,
        named_slots,
        ..
    } = context;
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

    if !props.has::<ContentBoxProps>() {
        props.write(ContentBoxProps {
            clipping: true,
            ..Default::default()
        });
    }

    let size_props = SizeBoxProps {
        width: SizeBoxSizeValue::Fill,
        height: SizeBoxSizeValue::Fill,
        ..Default::default()
    };

    let content = make_widget!(content_box)
        .key(key)
        .merge_props(props.clone())
        .listed_slot(
            make_widget!(button)
                .key("input-consumer")
                .with_props(NavItemActive)
                .named_slot(
                    "content",
                    make_widget!(size_box).key("size").with_props(size_props),
                ),
        )
        .listed_slot(
            make_widget!(nav_scroll_box_content)
                .key("content")
                .merge_props(content_props)
                .named_slot("content", content),
        )
        .listed_slot(scrollbars)
        .into();

    AreaBoxNode {
        id: id.to_owned(),
        slot: Box::new(content),
    }
    .into()
}

pub fn use_nav_scroll_box_side_scrollbars(context: &mut WidgetContext) {
    context.life_cycle.mount(|context| {
        let _ = context.state.write_with(SideScrollbarsState::default());
    });

    context.life_cycle.change(|context| {
        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<ButtonNotifyMessage>() {
                if msg.trigger_start() {
                    context.signals.write(NavSignal::Lock);
                }
                if msg.trigger_stop() {
                    context.signals.write(NavSignal::Unlock);
                }
                if msg.state.selected && (msg.state.trigger || msg.state.context) {
                    let mut dirty = false;
                    let mut data = context
                        .state
                        .read_cloned_or_default::<SideScrollbarsState>();
                    if msg.sender.key() == "hbar" {
                        data.horizontal = msg.state.pointer.x;
                        dirty = true;
                    } else if msg.sender.key() == "vbar" {
                        data.vertical = msg.state.pointer.y;
                        dirty = true;
                    }
                    if dirty {
                        let view = context.props.read_cloned_or_default::<ScrollBoxOwner>().0;
                        let pos = Vec2 {
                            x: data.horizontal,
                            y: data.vertical,
                        };
                        context.signals.write(NavSignal::Jump(NavJump::Scroll(
                            NavScroll::DirectFactor(view.into(), pos, false),
                        )));
                        let _ = context.state.write_with(data);
                    }
                }
            }
        }
    });
}

#[pre_hooks(
    use_nav_item_active,
    use_nav_container_active,
    use_nav_scroll_box_side_scrollbars
)]
pub fn nav_scroll_box_side_scrollbars(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext { id, key, props, .. } = context;

    let view_props = props.read_cloned_or_default::<ScrollViewState>();

    let SideScrollbarsProps {
        size,
        back_material,
        front_material,
    } = props.read_cloned_or_default();

    let hbar = if view_props.size_factor.x > 1.0 {
        let length = 1.0 / view_props.size_factor.y;
        let rest = 1.0 - length;

        let button_props = Props::new(ButtonNotifyProps(id.to_owned().into()))
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

        let front_props = Props::new(ImageBoxProps {
            material: front_material.clone(),
            ..Default::default()
        })
        .with(ContentBoxItemLayout {
            anchors: Rect {
                left: lerp(0.0, rest, view_props.value.x),
                right: lerp(length, 1.0, view_props.value.x),
                top: 0.0,
                bottom: 1.0,
            },
            ..Default::default()
        });

        let back = if let Some(material) = back_material.clone() {
            let props = ImageBoxProps {
                material,
                ..Default::default()
            };

            make_widget!(image_box).key("back").with_props(props).into()
        } else {
            WidgetNode::default()
        };

        make_widget!(button)
            .key("hbar")
            .merge_props(button_props)
            .named_slot(
                "content",
                make_widget!(content_box)
                    .key("container")
                    .listed_slot(back)
                    .listed_slot(
                        make_widget!(image_box)
                            .key("front")
                            .merge_props(front_props),
                    ),
            )
            .into()
    } else {
        WidgetNode::default()
    };

    let vbar = if view_props.size_factor.y > 1.0 {
        let length = 1.0 / view_props.size_factor.y;
        let rest = 1.0 - length;

        let button_props = Props::new(ButtonNotifyProps(id.to_owned().into()))
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

            make_widget!(image_box).key("back").with_props(props).into()
        } else {
            WidgetNode::default()
        };

        let front_props = Props::new(ImageBoxProps {
            material: front_material,
            ..Default::default()
        })
        .with(ContentBoxItemLayout {
            anchors: Rect {
                left: 0.0,
                right: 1.0,
                top: lerp(0.0, rest, view_props.value.y),
                bottom: lerp(length, 1.0, view_props.value.y),
            },
            ..Default::default()
        });

        make_widget!(button)
            .key("vbar")
            .merge_props(button_props)
            .named_slot(
                "content",
                make_widget!(content_box)
                    .key("container")
                    .listed_slot(back)
                    .listed_slot(
                        make_widget!(image_box)
                            .key("front")
                            .merge_props(front_props),
                    ),
            )
            .into()
    } else {
        WidgetNode::default()
    };

    make_widget!(content_box)
        .key(key)
        .listed_slot(hbar)
        .listed_slot(vbar)
        .into()
}
