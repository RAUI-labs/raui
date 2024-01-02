use raui::prelude::*;
#[allow(unused_imports)]
use raui_app::prelude::*;

fn main() {
    let tree = make_widget!(image_box).with_props(ImageBoxProps {
        material: ImageBoxMaterial::Image(ImageBoxImage {
            id: "./demos/in-game/resources/images/slider-background.png".to_owned(),
            // enable nine-slice by setting Frame scaling.
            scaling: ImageBoxImageScaling::Frame(ImageBoxFrame {
                // rectangle that describes margins of the frame of the source image texture.
                source: 3.0.into(),
                // rectangle that describes margins of the frame of the UI image being presented.
                destination: 64.0.into(),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    });

    DeclarativeApp::simple("Image Box - Frame", tree);
}
