use raui_app::app::declarative::DeclarativeApp;
use raui_core::{
    make_widget,
    widget::{
        component::{
            containers::{
                flex_box::{FlexBoxProps, flex_box},
                size_box::{SizeBoxProps, size_box},
            },
            image_box::{ImageBoxProps, image_box},
            text_box::{TextBoxProps, text_box},
        },
        unit::{
            flex::{FlexBoxDirection, FlexBoxItemLayout},
            image::{ImageBoxColor, ImageBoxMaterial, ImageBoxSizeValue},
            size::SizeBoxSizeValue,
            text::{TextBoxFont, TextBoxSizeValue},
        },
        utils::Color,
    },
};

fn main() {
    let tree = make_widget!(size_box)
        .with_props(SizeBoxProps {
            width: SizeBoxSizeValue::Fill,
            height: SizeBoxSizeValue::Content,
            ..Default::default()
        })
        .named_slot(
            "content",
            make_widget!(flex_box)
                .with_props(FlexBoxProps {
                    direction: FlexBoxDirection::VerticalTopToBottom,
                    ..Default::default()
                })
                .listed_slot(
                    make_widget!(text_box)
                        .with_props(FlexBoxItemLayout::no_growing_and_shrinking())
                        .with_props(TextBoxProps {
                            text: "Hello\nWorld!".to_owned(),
                            font: TextBoxFont {
                                name: "./demos/hello-world/resources/verdana.ttf".to_owned(),
                                size: 64.0,
                            },
                            color: Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            },
                            height: TextBoxSizeValue::Content,
                            ..Default::default()
                        }),
                )
                .listed_slot(
                    make_widget!(image_box)
                        .with_props(FlexBoxItemLayout::no_growing_and_shrinking())
                        .with_props(ImageBoxProps {
                            height: ImageBoxSizeValue::Exact(100.0),
                            material: ImageBoxMaterial::Color(ImageBoxColor {
                                color: Color {
                                    r: 1.0,
                                    g: 0.5,
                                    b: 0.0,
                                    a: 1.0,
                                },
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                )
                // this image should not be visible at all, a zero size layout.
                .listed_slot(
                    make_widget!(image_box).with_props(ImageBoxProps::colored(Color {
                        r: 0.5,
                        g: 0.5,
                        b: 0.5,
                        a: 1.0,
                    })),
                ),
        );

    DeclarativeApp::simple("Flex Box - Adaptive content size", tree);
}
