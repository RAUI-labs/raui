use raui::prelude::*;
#[allow(unused_imports)]
use raui_app::prelude::*;

fn main() {
    let tree = make_widget!(horizontal_box)
        .with_props(HorizontalBoxProps {
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
                    // basis sets exact width of the item.
                    basis: Some(100.0),
                    // weight of the item when its layout box has to grow in width.
                    grow: 0.5,
                    // weight of the item when its layout box has to shrink in width (0.0 means no shrinking).
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

    DeclarativeApp::simple("Horizontal Box", tree);
}
