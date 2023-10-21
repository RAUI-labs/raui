use raui::prelude::*;
#[allow(unused_imports)]
use raui_app::prelude::*;

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
                .with_props(FlexBoxItemLayout {
                    basis: Some(80.0),
                    grow: 0.0,
                    shrink: 0.0,
                    ..Default::default()
                })
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
                    ..Default::default()
                }),
        );

    DeclarativeApp::simple("RAUI application example", tree);
}
