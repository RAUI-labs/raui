use raui_app::app::declarative::DeclarativeApp;
use raui_core::{
    make_widget,
    widget::{
        component::{
            containers::horizontal_box::horizontal_box,
            image_box::{ImageBoxProps, image_box},
            space_box::{SpaceBoxProps, space_box},
        },
        unit::flex::FlexBoxItemLayout,
        utils::Color,
    },
};

fn main() {
    let tree = make_widget!(horizontal_box)
        .listed_slot(
            make_widget!(image_box).with_props(ImageBoxProps::colored(Color {
                r: 1.0,
                g: 0.25,
                b: 0.25,
                a: 1.0,
            })),
        )
        .listed_slot(
            make_widget!(space_box)
                // cube spacing means we set same separation both horizontally and vertically.
                .with_props(SpaceBoxProps::cube(64.0))
                // we set clear flex box layout to disallow space box fluidity.
                .with_props(FlexBoxItemLayout::cleared()),
        )
        .listed_slot(
            make_widget!(image_box).with_props(ImageBoxProps::colored(Color {
                r: 0.25,
                g: 0.25,
                b: 1.0,
                a: 1.0,
            })),
        );

    DeclarativeApp::simple("Space Box", tree);
}
