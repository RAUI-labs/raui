use raui_app::app::declarative::DeclarativeApp;
use raui_core::{
    make_widget,
    widget::{
        component::image_box::{ImageBoxProps, image_box},
        unit::image::{ImageBoxColor, ImageBoxMaterial},
        utils::Color,
    },
};

fn main() {
    let tree = make_widget!(image_box).with_props(ImageBoxProps {
        material: ImageBoxMaterial::Color(ImageBoxColor {
            color: Color {
                r: 1.0,
                g: 0.25,
                b: 0.25,
                a: 1.0,
            },
            ..Default::default()
        }),
        ..Default::default()
    });

    DeclarativeApp::simple("Image Box - Color", tree);
}
