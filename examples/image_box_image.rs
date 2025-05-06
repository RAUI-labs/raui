use raui_app::app::declarative::DeclarativeApp;
use raui_core::{
    make_widget,
    widget::{
        component::image_box::{ImageBoxProps, image_box},
        unit::image::{ImageBoxAspectRatio, ImageBoxImage, ImageBoxMaterial},
    },
};

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
            // if set to true then content instead of getting smaller to fit inside the layout box,
            // it will "leak" outside of the layout box.
            outside: true,
        }),
        ..Default::default()
    });

    DeclarativeApp::simple("Image Box - Image", tree);
}
