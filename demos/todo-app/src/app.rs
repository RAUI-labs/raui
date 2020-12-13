use crate::ui::components::app::app;
use ggez::{event::EventHandler, graphics, Context, GameResult};
use raui_core::{application::Application as UI, prelude::*};
use raui_ggez_renderer::{GgezRenderer, GgezResources};
use std::str::FromStr;

pub struct App {
    ui: UI,
    ui_resources: GgezResources,
}

impl App {
    pub fn new(ctx: &mut Context) -> Self {
        let mut ui_resources = GgezResources::default();
        ui_resources.fonts.insert(
            "verdana".to_owned(),
            graphics::Font::new(ctx, "/verdana.ttf").expect("GGEZ could not load `verdana.ttf`!"),
        );
        ui_resources.images.insert(
            "cat".to_owned(),
            graphics::Image::new(ctx, "/cat.jpg").expect("GGEZ could not load `cat.jpg`!"),
        );
        ui_resources.images.insert(
            "cats".to_owned(),
            graphics::Image::new(ctx, "/cats.jpg").expect("GGEZ could not load `cats.jpg`!"),
        );

        let mut ui = UI::new();
        let tree = widget! {{{
            FlexBox {
                id: WidgetId::from_str("app:/list").unwrap(),
                direction: FlexBoxDirection::VerticalTopToBottom,
                // separation: 16.0,
                items: vec![
                    FlexBoxItem {
                        slot: TextBox {
                            id: WidgetId::from_str("app:/text").unwrap(),
                            text: "Hello, World!".to_owned(),
                            alignment: TextBoxAlignment::Center,
                            font: TextBoxFont {
                                name: "verdana".to_owned(),
                                size: 48.0,
                                ..Default::default()
                            },
                            color: Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
                            ..Default::default()
                        }.into(),
                        basis: Some(64.0),
                        fill: 1.0,
                        // TODO: THIS CAUSES PROBLEMS.
                        // margin: Rect {
                        //     left: 8.0,
                        //     right: 8.0,
                        //     top: 8.0,
                        //     bottom: 8.0,
                        // },
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
            // ContentBox {
            //     id: WidgetId::from_str("app:/content").unwrap(),
            //     items: vec![
            //         ContentBoxItem {
            //             layout: ContentBoxItemLayout {
            //                 anchors: Rect {
            //                     left: 0.0,
            //                     right: 1.0,
            //                     top: 0.0,
            //                     bottom: 1.0,
            //                 },
            //                 margin: Rect {
            //                     left: 32.0,
            //                     right: 32.0,
            //                     top: 32.0,
            //                     bottom: 32.0,
            //                 },
            //                 ..Default::default()
            //             },
            //             slot: ,
            //         },
            //         ContentBoxItem {
            //             layout: ContentBoxItemLayout {
            //                 anchors: Rect {
            //                     left: 0.0,
            //                     right: 1.0,
            //                     top: 0.0,
            //                     bottom: 1.0,
            //                 },
            //                 margin: Rect {
            //                     left: 64.0,
            //                     right: 64.0,
            //                     top: 64.0,
            //                     bottom: 64.0,
            //                 },
            //                 ..Default::default()
            //             },
            //             slot: ,
            //         },
            //     ],
            //     ..Default::default()
            // }
        }}};
        ui.apply(tree);
        Self { ui, ui_resources }
    }
}

impl EventHandler for App {
    fn update(&mut self, _: &mut Context) -> GameResult<()> {
        self.ui.process();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::WHITE);
        let (width, height) = graphics::drawable_size(ctx);
        let ui_space = Rect {
            left: 0.0,
            right: width,
            top: 0.0,
            bottom: height,
        };
        self.ui
            .layout(ui_space, &mut DefaultLayoutEngine)
            .expect("UI could not layout widgets!");
        self.ui
            .render(&mut GgezRenderer::new(ctx, &mut self.ui_resources))
            .expect("GGEZ renderer could not render UI!");
        graphics::present(ctx)
    }
}
