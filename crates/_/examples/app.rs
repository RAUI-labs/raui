use raui_app::app::declarative::DeclarativeApp;
use raui_core::{
    make_widget,
    widget::{
        component::{
            containers::{flex_box::FlexBoxProps, vertical_box::vertical_box},
            image_box::{ImageBoxProps, image_box},
            text_box::{TextBoxProps, text_box},
        },
        unit::{
            flex::FlexBoxItemLayout,
            text::{TextBoxFont, TextBoxHorizontalAlign, TextBoxSizeValue},
        },
        utils::Color,
    },
};

fn main() {
    let tree = make_widget!(vertical_box)
        .with_props(FlexBoxProps {
            separation: 50.0,
            ..Default::default()
        })
        .listed_slot(
            make_widget!(image_box).with_props(ImageBoxProps::image_aspect_ratio(
                "./demos/hello-world/resources/cats.jpg",
                false,
            )),
        )
        .listed_slot(
            make_widget!(text_box)
                .with_props(FlexBoxItemLayout::no_growing_and_shrinking())
                .with_props(TextBoxProps {
                    text: "RAUI application example".to_owned(),
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
                    horizontal_align: TextBoxHorizontalAlign::Center,
                    height: TextBoxSizeValue::Content,
                    ..Default::default()
                }),
        );

    DeclarativeApp::simple("RAUI application example", tree);
}
