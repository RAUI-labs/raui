use raui::prelude::*;
#[allow(unused_imports)]
use raui_app::prelude::*;

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
