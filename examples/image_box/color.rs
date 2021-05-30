use raui::prelude::*;
use raui_quick_start::RauiQuickStartBuilder;

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

    RauiQuickStartBuilder::default()
        .window_title("Image Box - Color".to_owned())
        .widget_tree(tree.into())
        .build()
        .unwrap()
        .run()
        .unwrap();
}
