use raui::prelude::*;
use raui_quick_start::RauiQuickStartBuilder;

fn main() {
    let tree = make_widget!(text_box).with_props(TextBoxProps {
        text: "RAUI text box example".to_owned(),
        font: TextBoxFont {
            name: "./demos/hello-world/resources/verdana.ttf".to_owned(),
            size: 64.0,
        },
        color: Color {
            r: 0.0,
            g: 0.0,
            b: 0.5,
            a: 1.0,
        },
        ..Default::default()
    });

    RauiQuickStartBuilder::default()
        .window_title("Text Box".to_owned())
        .widget_tree(tree.into())
        .build()
        .unwrap()
        .run()
        .unwrap();
}
