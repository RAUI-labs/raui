use crate::ui::components::{
    color_rect::{color_rect, ColorRectProps},
    image_button::{image_button, ImageButtonProps},
};
use raui_core::prelude::*;

widget_component! {
    pub content(key, props) {
        let props = props.clone().with(GridBoxProps {
            cols: 2,
            rows: 2,
            ..Default::default()
        });

        let props0 = Props::new(ImageButtonProps {
            id: "cat".to_owned(),
            horizontal_alignment: 1.0,
        })
        .with(GridBoxItemLayout {
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
        });

        let props1 = Props::new(ColorRectProps {
            color: Color { r: 1.0, g: 0.0, b: 0.0, a: 0.5 },
        })
        .with(GridBoxItemLayout {
            space_occupancy: IntRect {
                left: 1,
                right: 2,
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
        });

        let props2 = Props::new(ImageButtonProps {
            id: "cats".to_owned(),
            horizontal_alignment: 0.5,
        })
        .with(GridBoxItemLayout {
            space_occupancy: IntRect {
                left: 0,
                right: 2,
                top: 1,
                bottom: 2,
            },
            margin: Rect {
                left: 8.0,
                right: 8.0,
                top: 8.0,
                bottom: 8.0,
            },
            ..Default::default()
        });

        widget! {
            (#{key} grid_box: {props} [
                (image_button: {props0})
                (color_rect: {props1})
                (image_button: {props2})
            ])
        }
    }
}
