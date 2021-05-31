use raui::prelude::*;
use raui_quick_start::RauiQuickStartBuilder;

fn main() {
    let tree = make_widget!(wrap_box)
        .with_props(WrapBoxProps {
            margin: 50.0.into(),
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

    RauiQuickStartBuilder::default()
        .window_title("Wrap Box".to_owned())
        .widget_tree(tree.into())
        .build()
        .unwrap()
        .run()
        .unwrap();
}
