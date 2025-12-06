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
                navigation::{NavItemActive, use_nav_container_active},
                scroll_view::ScrollViewRange,
            },
            text_box::{TextBoxProps, text_box},
        },
        context::WidgetContext,
        node::WidgetNode,
        unit::{
            flex::FlexBoxItemLayout,
            image::{
                ImageBoxAspectRatio, ImageBoxColor, ImageBoxImage, ImageBoxMaterial,
                ImageBoxSizeValue,
            },
            size::SizeBoxSizeValue,
            text::{TextBoxFont, TextBoxSizeValue},
        },
        utils::{Color, Rect},
    },
};

#[pre_hooks(use_nav_container_active)]
fn app(mut ctx: WidgetContext) -> WidgetNode {
    make_widget!(wrap_box)
        .with_props(WrapBoxProps {
            margin: Rect {
                left: 100.0,
                right: 50.0,
                top: 75.0,
                bottom: 25.0,
            },
            ..Default::default()
        })
        .named_slot(
            "content",
            make_widget!(nav_scroll_box)
                .with_props(NavItemActive)
                .with_props(ScrollViewRange::default())
                .named_slot(
                    "content",
                    make_widget!(size_box)
                        .with_props(SizeBoxProps {
                            width: SizeBoxSizeValue::Fill,
                            // first we make sure to put content in size box that
                            // uses content height to make it take minimal space.
                            height: SizeBoxSizeValue::Content,
                            ..Default::default()
                        })
                        .named_slot(
                            "content",
                            make_widget!(vertical_box)
                                .with_props(VerticalBoxProps {
                                    // we need to make sure all items are not
                                    // growing and shrinking to let size box
                                    // calculate correct content size. growing
                                    // and shrinking would make items take all
                                    // available space, filling all container.
                                    override_slots_layout: Some(
                                        FlexBoxItemLayout::no_growing_and_shrinking(),
                                    ),
                                    ..Default::default()
                                })
                                .listed_slot(make_widget!(image_box).with_props(ImageBoxProps {
                                    height: ImageBoxSizeValue::Exact(300.0),
                                    material: ImageBoxMaterial::Image(ImageBoxImage {
                                        id: "./crates/_/examples/resources/map.png".to_owned(),
                                        ..Default::default()
                                    }),
                                    content_keep_aspect_ratio: Some(ImageBoxAspectRatio {
                                        horizontal_alignment: 0.5,
                                        vertical_alignment: 0.5,
                                        outside: false,
                                    }),
                                    ..Default::default()
                                }))
                                .listed_slot(make_widget!(text_box).with_props(TextBoxProps {
                                    text: include_str!("./resources/long_text.txt").to_owned(),
                                    font: TextBoxFont {
                                        name:
                                            "./demos/hello-world/resources/verdana.ttf".to_owned(),
                                        size: 64.0,
                                    },
                                    color: Color {
                                        r: 0.0,
                                        g: 0.0,
                                        b: 0.5,
                                        a: 1.0,
                                    },
                                    height: TextBoxSizeValue::Content,
                                    ..Default::default()
                                })),
                        ),
                )
                .named_slot(
                    "scrollbars",
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

fn main() {
    DeclarativeApp::simple("Scroll Box - Adaptive content size", make_widget!(app));
}
