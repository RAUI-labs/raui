use raui::prelude::*;
#[allow(unused_imports)]
use raui_app::prelude::*;

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
                    make_widget!(text_box).with_props(TextBoxProps {
                        text: include_str!("./resources/long_text.txt").to_owned(),
                        font: TextBoxFont {
                            name: "./demos/hello-world/resources/verdana.ttf".to_owned(),
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
                    }),
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
