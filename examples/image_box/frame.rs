use raui::prelude::*;
use raui_quick_start::RauiQuickStartBuilder;

fn main() {
    let tree = make_widget!(image_box).with_props(ImageBoxProps {
        material: ImageBoxMaterial::Image(ImageBoxImage {
            id: "./demos/in-game/resources/images/panel.png".to_owned(),
            // enable nine-slice by setting Frame scaling.
            scaling: ImageBoxImageScaling::Frame(ImageBoxFrame {
                // rectangle that describes margins of the frame of the source image texture.
                source: 6.0.into(),
                // rectangle that describes margins of the frame of the UI image being presented.
                destination: 64.0.into(),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    });

    RauiQuickStartBuilder::default()
        .window_title("Image Box - Frame".to_owned())
        .widget_tree(tree.into())
        .build()
        .unwrap()
        .run()
        .unwrap();
}
