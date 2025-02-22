// Make sure you have seen `size_box` code example first, because this is an evolution of that.

use raui::prelude::*;
#[allow(unused_imports)]
use raui_app::prelude::*;

fn main() {
    let tree = make_widget!(size_box)
        .with_props(SizeBoxProps {
            width: SizeBoxSizeValue::Fill,
            height: SizeBoxSizeValue::Fill,
            // enforce width to be percentage of height.
            keep_aspect_ratio: SizeBoxAspectRatio::WidthOfHeight(0.5),
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

    DeclarativeApp::simple("Size Box - Keep Aspect Ratio", tree);
}
