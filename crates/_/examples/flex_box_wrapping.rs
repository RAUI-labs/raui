use raui_app::app::declarative::DeclarativeApp;
use raui_core::{
    make_widget,
    widget::{
        component::{
            containers::{
                flex_box::{FlexBoxProps, flex_box},
                size_box::{SizeBoxProps, size_box},
            },
            image_box::{ImageBoxProps, image_box},
        },
        unit::{
            flex::{FlexBoxDirection, FlexBoxItemLayout},
            size::SizeBoxSizeValue,
        },
        utils::Color,
    },
};

fn main() {
    let tree = make_widget!(flex_box)
        .with_props(FlexBoxProps {
            direction: FlexBoxDirection::VerticalTopToBottom,
            // Wrapping makes children fit into multiple rows/columns.
            wrap: true,
            ..Default::default()
        })
        .listed_slots((0..18).map(|_| {
            make_widget!(size_box)
                .with_props(FlexBoxItemLayout::cleared())
                .with_props(SizeBoxProps {
                    width: SizeBoxSizeValue::Exact(100.0),
                    height: SizeBoxSizeValue::Exact(100.0),
                    margin: 20.0.into(),
                    ..Default::default()
                })
                .named_slot(
                    "content",
                    make_widget!(image_box).with_props(ImageBoxProps::colored(Color {
                        r: 0.25,
                        g: 0.25,
                        b: 0.25,
                        a: 1.0,
                    })),
                )
        }));

    DeclarativeApp::simple("Flex Box - Wrapping content", tree);
}
