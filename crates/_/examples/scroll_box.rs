use raui_app::app::declarative::DeclarativeApp;
use raui_core::{
    make_widget, pre_hooks,
    widget::{
        component::{
            containers::{
                scroll_box::{SideScrollbarsProps, nav_scroll_box, nav_scroll_box_side_scrollbars},
                size_box::{SizeBoxProps, size_box},
                vertical_box::{VerticalBoxProps, vertical_box},
                wrap_box::{WrapBoxProps, wrap_box},
            },
            image_box::{ImageBoxProps, image_box},
            interactive::{
                button::{ButtonNotifyMessage, ButtonNotifyProps, button},
                navigation::{NavItemActive, use_nav_container_active},
                scroll_view::ScrollViewRange,
            },
        },
        context::WidgetContext,
        node::WidgetNode,
        unit::{
            flex::FlexBoxItemLayout,
            image::{ImageBoxColor, ImageBoxMaterial},
            size::SizeBoxSizeValue,
        },
        utils::{Color, Rect},
    },
};

// we make this root widget a navigable container to let scrol box perform scrolling.
#[pre_hooks(use_nav_container_active)]
fn app(mut ctx: WidgetContext) -> WidgetNode {
    make_widget!(wrap_box)
        .with_props(WrapBoxProps {
            margin: Rect {
                left: 50.0,
                right: 50.0,
                top: 75.0,
                bottom: 25.0,
            },
            ..Default::default()
        })
        .named_slot(
            "content",
            make_widget!(nav_scroll_box)
                // we activate scroll box navigation - it is disabled by default.
                .with_props(NavItemActive)
                // apply scroll view range to limit scrolling area (without it you could scroll infinitely).
                .with_props(ScrollViewRange::default())
                .named_slot(
                    "content",
                    // typical use of scroll box is to wrap around some kind of list but we can actually
                    // put there anything and scroll box will scroll that content.
                    make_widget!(vertical_box)
                        .with_props(VerticalBoxProps {
                            override_slots_layout: Some(FlexBoxItemLayout {
                                grow: 0.0,
                                shrink: 0.0,
                                ..Default::default()
                            }),
                            ..Default::default()
                        })
                        .listed_slot(make_widget!(item).key(0).with_props(true))
                        .listed_slot(make_widget!(item).key(1).with_props(false))
                        .listed_slot(make_widget!(item).key(2).with_props(true))
                        .listed_slot(make_widget!(item).key(3).with_props(false))
                        .listed_slot(make_widget!(item).key(4).with_props(true))
                        .listed_slot(make_widget!(item).key(5).with_props(false)),
                )
                .named_slot(
                    "scrollbars",
                    // scrollbars used here are side buttons that you can drag to scroll content on
                    // separate axes, but you could make a custom scrollbars component that for example
                    // uses single button that allows to scroll in both axes at once with dragging.
                    make_widget!(nav_scroll_box_side_scrollbars).with_props(SideScrollbarsProps {
                        size: 20.0,
                        back_material: Some(ImageBoxMaterial::Color(ImageBoxColor {
                            color: Color {
                                r: 0.15,
                                g: 0.15,
                                b: 0.15,
                                a: 1.0,
                            },
                            ..Default::default()
                        })),
                        front_material: ImageBoxMaterial::Color(ImageBoxColor {
                            color: Color {
                                r: 0.85,
                                g: 0.85,
                                b: 0.85,
                                a: 1.0,
                            },
                            ..Default::default()
                        }),
                    }),
                ),
        )
        .into()
}

fn use_item(ctx: &mut WidgetContext) {
    ctx.life_cycle.change(|ctx| {
        for msg in ctx.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<ButtonNotifyMessage>()
                && msg.trigger_start()
            {
                println!("Button clicked: {:?}", msg.sender.key());
            }
        }
    });
}

#[pre_hooks(use_item)]
fn item(mut ctx: WidgetContext) -> WidgetNode {
    let color = if ctx.props.read_cloned_or_default::<bool>() {
        Color {
            r: 0.5,
            g: 0.5,
            b: 0.5,
            a: 1.0,
        }
    } else {
        Color {
            r: 0.25,
            g: 0.25,
            b: 0.25,
            a: 1.0,
        }
    };

    make_widget!(button)
        .with_props(NavItemActive)
        .with_props(ButtonNotifyProps(ctx.id.to_owned().into()))
        .named_slot(
            "content",
            make_widget!(size_box)
                .with_props(SizeBoxProps {
                    width: SizeBoxSizeValue::Fill,
                    height: SizeBoxSizeValue::Exact(136.0),
                    ..Default::default()
                })
                .named_slot(
                    "content",
                    make_widget!(image_box).with_props(ImageBoxProps::colored(color)),
                ),
        )
        .into()
}

fn main() {
    DeclarativeApp::simple("Scroll Box", make_widget!(app));
}
