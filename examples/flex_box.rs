use raui::prelude::*;
use raui_quick_start::RauiQuickStartBuilder;

fn main() {
    let tree = make_widget!(flex_box)
        .with_props(FlexBoxProps {
            direction: FlexBoxDirection::VerticalBottomToTop,
            separation: 50.0,
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
                .with_props(FlexBoxItemLayout {
                    // basis sets exact size of the item in main axis.
                    basis: Some(100.0),
                    // weight of the item when its layout box has to grow.
                    grow: 0.5,
                    // weight of the item when its layout box has to shrink (0.0 means no shrinking).
                    shrink: 0.0,
                    ..Default::default()
                }),
        )
        .listed_slot(
            make_widget!(image_box).with_props(ImageBoxProps::colored(Color {
                r: 0.25,
                g: 1.0,
                b: 0.25,
                a: 1.0,
            })),
        )
        .listed_slot(
            make_widget!(image_box)
                .with_props(ImageBoxProps::colored(Color {
                    r: 0.25,
                    g: 0.25,
                    b: 1.0,
                    a: 1.0,
                }))
                .with_props(FlexBoxItemLayout {
                    basis: Some(100.0),
                    grow: 0.0,
                    shrink: 0.5,
                    ..Default::default()
                }),
        );

    RauiQuickStartBuilder::default()
        .window_title("Flex Box".to_owned())
        .widget_tree(tree.into())
        .build()
        .unwrap()
        .run()
        .unwrap();
}
