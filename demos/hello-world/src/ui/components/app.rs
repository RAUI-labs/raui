use raui_core::prelude::*;
use std::{convert::TryInto, str::FromStr};

widget_component! {
    pub app(named_slots) {
        unpack_named_slots!(named_slots => { title });

        widget! {{{
            FlexBox {
                id: WidgetId::from_str("app:/list").unwrap(),
                direction: FlexBoxDirection::VerticalTopToBottom,
                separation: 16.0,
                items: vec![
                    FlexBoxItem {
                        slot: title.try_into().unwrap(),
                        basis: Some(64.0),
                        fill: 1.0,
                        margin: Rect {
                            left: 8.0,
                            right: 8.0,
                            top: 8.0,
                            bottom: 8.0,
                        },
                        ..Default::default()
                    }.into(),
                    FlexBoxItem {
                        slot: GridBox {
                            id: WidgetId::from_str("app:/grid").unwrap(),
                            cols: 2,
                            rows: 2,
                            items: vec![
                                GridBoxItem {
                                    slot: ImageBox {
                                        id: WidgetId::from_str("app:/0").unwrap(),
                                        material: ImageBoxMaterial::Image(ImageBoxImage {
                                            id: "cat".to_owned(),
                                            ..Default::default()
                                        }),
                                        content_keep_aspect_ratio: Some(ImageBoxAspectRatio {
                                            horizontal_alignment: 1.0,
                                            vertical_alignment: 0.5
                                        }),
                                        ..Default::default()
                                    }.into(),
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
                                }.into(),
                                GridBoxItem {
                                    slot: ImageBox {
                                        id: WidgetId::from_str("app:/1").unwrap(),
                                        material: ImageBoxMaterial::Color(Color { r: 0.5, g: 0.1, b: 0.1, a: 0.9 }),
                                        ..Default::default()
                                    }.into(),
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
                                }.into(),
                                GridBoxItem {
                                    slot: ImageBox {
                                        id: WidgetId::from_str("app:/2").unwrap(),
                                        material: ImageBoxMaterial::Image(ImageBoxImage {
                                            id: "cats".to_owned(),
                                            ..Default::default()
                                        }),
                                        content_keep_aspect_ratio: Some(ImageBoxAspectRatio {
                                            horizontal_alignment: 0.5,
                                            vertical_alignment: 0.5
                                        }),
                                        ..Default::default()
                                    }.into(),
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
                                }.into(),
                            ],
                            ..Default::default()
                        }.into(),
                        grow: 1.0,
                        fill: 1.0,
                        margin: Rect {
                            left: 8.0,
                            right: 8.0,
                            top: 8.0,
                            bottom: 8.0,
                        },
                        ..Default::default()
                    }.into(),
                ],
                ..Default::default()
            }
        }}}
    }
}
