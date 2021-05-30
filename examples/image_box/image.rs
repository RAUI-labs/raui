use raui::prelude::*;
use raui_quick_start::RauiQuickStartBuilder;

fn main() {
    let tree = make_widget!(image_box).with_props(ImageBoxProps {
        material: ImageBoxMaterial::Image(ImageBoxImage {
            id: "./demos/hello-world/resources/cats.jpg".to_owned(),
            ..Default::default()
        }),
        // makes internal image size keeping its aspect ratio.
        content_keep_aspect_ratio: Some(ImageBoxAspectRatio {
            // horizontal alignment of the content relative to the horizontal free space.
            horizontal_alignment: 0.5,
            // vertical alignment of the content relative to the vertical free space.
            vertical_alignment: 0.5,
        }),
        ..Default::default()
    });

    RauiQuickStartBuilder::default()
        .window_title("Image Box - Image".to_owned())
        .widget_tree(tree.into())
        .build()
        .unwrap()
        .run()
        .unwrap();
}
