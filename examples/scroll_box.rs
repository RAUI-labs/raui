use raui::prelude::*;
use raui_quick_start::RauiQuickStartBuilder;

// we make this root widget a navigable container to let scrol box perform scrolling.
#[pre_hooks(use_nav_container_active)]
fn app(mut ctx: WidgetContext) -> WidgetNode {
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
                .listed_slot(
                    make_widget!(item)
                        .with_props(true)
                        .with_props(FlexBoxItemLayout {
                            grow: 0.0,
                            shrink: 0.0,
                            ..Default::default()
                        }),
                )
                .listed_slot(
                    make_widget!(item)
                        .with_props(false)
                        .with_props(FlexBoxItemLayout {
                            grow: 0.0,
                            shrink: 0.0,
                            ..Default::default()
                        }),
                )
                .listed_slot(
                    make_widget!(item)
                        .with_props(true)
                        .with_props(FlexBoxItemLayout {
                            grow: 0.0,
                            shrink: 0.0,
                            ..Default::default()
                        }),
                )
                .listed_slot(
                    make_widget!(item)
                        .with_props(false)
                        .with_props(FlexBoxItemLayout {
                            grow: 0.0,
                            shrink: 0.0,
                            ..Default::default()
                        }),
                )
                .listed_slot(
                    make_widget!(item)
                        .with_props(true)
                        .with_props(FlexBoxItemLayout {
                            grow: 0.0,
                            shrink: 0.0,
                            ..Default::default()
                        }),
                )
                .listed_slot(
                    make_widget!(item)
                        .with_props(false)
                        .with_props(FlexBoxItemLayout {
                            grow: 0.0,
                            shrink: 0.0,
                            ..Default::default()
                        }),
                ),
        )
        .named_slot(
            "scrollbars",
            // scrollbars used here are side buttons that you can drag to scroll content on
            // separate axes, but you could make a custom scrollbars component that for example
            // uses single button that allows to scroll inboth axes at once.
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
        )
        .into()
}

fn item(ctx: WidgetContext) -> WidgetNode {
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

    make_widget!(size_box)
        .with_props(SizeBoxProps {
            width: SizeBoxSizeValue::Fill,
            height: SizeBoxSizeValue::Exact(136.0),
            ..Default::default()
        })
        .named_slot(
            "content",
            make_widget!(image_box).with_props(ImageBoxProps::colored(color)),
        )
        .into()
}

fn main() {
    RauiQuickStartBuilder::default()
        .window_title("Scroll Box".to_owned())
        .widget_tree(make_widget!(app).into())
        .build()
        .unwrap()
        .run()
        .unwrap();
}
