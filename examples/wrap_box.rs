use raui::prelude::*;
#[allow(unused_imports)]
use raui_app::prelude::*;

fn main() {
    let tree = make_widget!(wrap_box)
        .with_props(WrapBoxProps {
            // wrap box just wraps its content with margin.
            margin: 50.0.into(),
            // by default it wraps around content using its size, but we can make it fill whole
            // available space and then put content inside the space with margins.
            fill: true,
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
