use raui::prelude::*;
#[allow(unused_imports)]
use raui_app::prelude::*;

fn main() {
    let tree = make_widget!(flex_box)
        .with_props(FlexBoxProps {
            direction: FlexBoxDirection::VerticalBottomToTop,
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
                    // percentage of the item size in cross axis (here how much of horizontal space it fills).
                    fill: 0.75,
                    // tells how much to which side item is aligned when there is free space available.
                    align: 1.0,
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
                .with_props(FlexBoxItemLayout {
                    margin: 10.0.into(),
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
                .with_props(FlexBoxItemLayout {
                    basis: Some(100.0),
                    grow: 0.0,
                    shrink: 0.5,
                    fill: 0.5,
                    align: 0.5,
                    ..Default::default()
                }),
        );

    DeclarativeApp::simple("Flex Box", tree);
}
