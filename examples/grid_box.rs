use raui::prelude::*;
use raui_quick_start::RauiQuickStartBuilder;

fn main() {
    let tree = make_widget!(grid_box)
        .with_props(GridBoxProps {
            cols: 2,
            rows: 2,
            ..Default::default()
        })
        .listed_slot(
            make_widget!(image_box)
                .with_props(ImageBoxProps::colored(Color {
                    r: 1.0,
                    g: 0.25,
                    b: 0.25,
                    a: 1.0,
                }))
                .with_props(GridBoxItemLayout {
                    space_occupancy: IntRect {
                        left: 0,
                        right: 1,
                        top: 0,
                        bottom: 1,
                    },
                    ..Default::default()
                }),
        )
        .listed_slot(
            make_widget!(image_box)
                .with_props(ImageBoxProps::colored(Color {
                    r: 0.25,
                    g: 1.0,
                    b: 0.25,
                    a: 1.0,
                }))
                .with_props(GridBoxItemLayout {
                    space_occupancy: IntRect {
                        left: 1,
                        right: 2,
                        top: 0,
                        bottom: 1,
                    },
                    ..Default::default()
                }),
        )
        .listed_slot(
            make_widget!(image_box)
                .with_props(ImageBoxProps::colored(Color {
                    r: 0.25,
                    g: 0.25,
                    b: 1.0,
                    a: 1.0,
                }))
                .with_props(GridBoxItemLayout {
                    space_occupancy: IntRect {
                        left: 0,
                        right: 2,
                        top: 1,
                        bottom: 2,
                    },
                    ..Default::default()
                }),
        );

    RauiQuickStartBuilder::default()
        .window_title("Grid Box".to_owned())
        .widget_tree(tree.into())
        .build()
        .unwrap()
        .run()
        .unwrap();
}
