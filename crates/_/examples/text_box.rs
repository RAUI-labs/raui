use raui_app::app::declarative::DeclarativeApp;
use raui_core::{
    make_widget,
    widget::{
        component::text_box::{TextBoxProps, text_box},
        unit::text::{TextBoxFont, TextBoxHorizontalAlign, TextBoxVerticalAlign},
        utils::Color,
    },
};

fn main() {
    let tree = make_widget!(text_box).with_props(TextBoxProps {
        text: "RAUI\nText Box".to_owned(),
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
        vertical_align: TextBoxVerticalAlign::Middle,
        ..Default::default()
    });

    DeclarativeApp::simple("Text Box", tree);
}
