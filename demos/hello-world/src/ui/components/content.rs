use crate::ui::components::{
    color_rect::{ColorRectProps, color_rect},
    image_button::{ImageButtonProps, image_button},
};
use raui::core::{
    make_widget,
    widget::{
        component::containers::grid_box::{GridBoxProps, grid_box},
        context::WidgetContext,
        node::WidgetNode,
        unit::grid::GridBoxItemLayout,
        utils::{Color, IntRect, Rect},
    },
};

pub fn content(context: WidgetContext) -> WidgetNode {
    let WidgetContext { key, props, .. } = context;

    make_widget!(grid_box)
        .key(key)
        .merge_props(props.clone())
        .with_props(GridBoxProps {
            cols: 2,
            rows: 2,
            ..Default::default()
        })
        .listed_slot(
            make_widget!(image_button)
                .with_props(ImageButtonProps {
                    image: "./resources/cat.jpg".to_owned(),
                    horizontal_alignment: 1.0,
                })
                .with_props(GridBoxItemLayout {
                    space_occupancy: IntRect {
                        left: 0,
                        right: 1,
                        top: 0,
                        bottom: 1,
                    },
                    margin: Rect {
                        left: 8.0,
                        right: 8.0,
                        top: 8.0,
                        bottom: 8.0,
                    },
                    ..Default::default()
                }),
        )
        .listed_slot(
            make_widget!(color_rect)
                .with_props(ColorRectProps {
                    color: Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.5,
                    },
                })
                .with_props(GridBoxItemLayout {
                    space_occupancy: IntRect {
                        left: 1,
                        right: 2,
                        top: 0,
                        bottom: 1,
                    },
                    margin: 8.0.into(),
                    ..Default::default()
                }),
        )
        .listed_slot(
            make_widget!(image_button)
                .with_props(ImageButtonProps {
                    image: "./resources/cats.jpg".to_owned(),
                    horizontal_alignment: 0.5,
                })
                .with_props(GridBoxItemLayout {
                    space_occupancy: IntRect {
                        left: 0,
                        right: 2,
                        top: 1,
                        bottom: 2,
                    },
                    margin: 8.0.into(),
                    ..Default::default()
                }),
        )
        .into()
}
