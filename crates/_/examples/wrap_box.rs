use raui_app::app::declarative::DeclarativeApp;
use raui_core::{
    make_widget,
    widget::{
        component::{
            containers::wrap_box::{WrapBoxProps, wrap_box},
            image_box::{ImageBoxProps, image_box},
        },
        utils::Color,
    },
};

fn main() {
    let tree = make_widget!(wrap_box)
        .with_props(WrapBoxProps {
            // wrap box just wraps its content with margin.
            margin: 50.0.into(),
            ..Default::default()
        })
        .named_slot(
            "content",
            make_widget!(image_box).with_props(ImageBoxProps::colored(Color {
                r: 1.0,
                g: 0.25,
                b: 0.25,
                a: 1.0,
            })),
        );

    DeclarativeApp::simple("Wrap Box", tree);
}
