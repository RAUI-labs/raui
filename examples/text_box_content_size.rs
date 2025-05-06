use raui_app::app::declarative::DeclarativeApp;
use raui_core::{
    make_widget,
    widget::{
        component::{
            containers::horizontal_box::horizontal_box,
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
    let tree = make_widget!(horizontal_box)
        .listed_slot(
            make_widget!(text_box)
                .with_props(FlexBoxItemLayout {
                    // Disable growing and shrinking of the text box to allow it to
                    // take the size of its content in the list.
                    grow: 0.0,
                    shrink: 0.0,
                    margin: 20.0.into(),
                    ..Default::default()
                })
                .with_props(TextBoxProps {
                    text: "RAUI\nContent Size".to_owned(),
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
                    horizontal_align: TextBoxHorizontalAlign::Right,
                    // Setting text size to its content allows for fitting other
                    // widgets nicely around that text box.
                    width: TextBoxSizeValue::Content,
                    height: TextBoxSizeValue::Content,
                    ..Default::default()
                }),
        )
        .listed_slot(
            make_widget!(image_box).with_props(ImageBoxProps::colored(Color {
                r: 0.5,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            })),
        );

    DeclarativeApp::simple("Text Box - Content Size", tree);
}
