use super::{inventory::inventory, quests::quests, settings::settings};
use crate::model::menu::{Menu, MenuScreen};
use raui::{
    core::{
        make_widget, pre_hooks,
        widget::{
            component::{
                containers::content_box::content_box,
                image_box::{ImageBoxProps, image_box},
            },
            context::WidgetContext,
            node::WidgetNode,
            unit::{
                image::{
                    ImageBoxAspectRatio, ImageBoxFrame, ImageBoxImage, ImageBoxImageScaling,
                    ImageBoxMaterial,
                },
                text::{TextBoxFont, TextBoxHorizontalAlign, TextBoxVerticalAlign},
            },
            utils::Color,
        },
    },
    material::theme::{
        ThemeColorSet, ThemeColors, ThemeColorsBundle, ThemeProps, ThemedButtonMaterial,
        ThemedImageMaterial, ThemedSliderMaterial, ThemedSwitchMaterial, ThemedTextMaterial,
        new_all_white_theme,
    },
};

fn use_app(context: &mut WidgetContext) {
    context.life_cycle.mount(|mut context| {
        context
            .view_models
            .bindings(Menu::VIEW_MODEL, Menu::SCREEN)
            .unwrap()
            .bind(context.id.to_owned());
    });
}

#[pre_hooks(use_app)]
pub fn app(mut context: WidgetContext) -> WidgetNode {
    let menu = context
        .view_models
        .view_model(Menu::VIEW_MODEL)
        .unwrap()
        .read::<Menu>()
        .unwrap();

    make_widget!(content_box)
        .key("screen")
        .with_shared_props(make_theme())
        // Let's pretend this image is underlying game world.
        .listed_slot(
            make_widget!(image_box)
                .key("game")
                .with_props(ImageBoxProps {
                    material: ImageBoxMaterial::Image(ImageBoxImage {
                        id: "resources/images/game-mockup.png".to_owned(),
                        ..Default::default()
                    }),
                    content_keep_aspect_ratio: Some(ImageBoxAspectRatio {
                        horizontal_alignment: 0.5,
                        vertical_alignment: 0.5,
                        outside: true,
                    }),
                    ..Default::default()
                }),
        )
        // And this is our actual game UI screens.
        .maybe_listed_slot(match *menu.screen {
            MenuScreen::None => None,
            MenuScreen::Settings => Some(make_widget!(settings)),
            MenuScreen::Inventory => Some(make_widget!(inventory)),
            MenuScreen::Quests => Some(make_widget!(quests)),
        })
        .into()
}

fn make_theme() -> ThemeProps {
    new_all_white_theme()
        .background_colors(ThemeColorsBundle::uniform(ThemeColors::uniform(
            ThemeColorSet::uniform(Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.7,
            }),
        )))
        .button_background(
            "",
            ThemedButtonMaterial {
                default: ThemedImageMaterial::Image(ImageBoxImage {
                    id: "resources/images/button-default.png".to_owned(),
                    scaling: ImageBoxImageScaling::Frame(ImageBoxFrame {
                        source: 6.0.into(),
                        destination: 6.0.into(),
                        frame_only: false,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                selected: ThemedImageMaterial::Image(ImageBoxImage {
                    id: "resources/images/button-selected.png".to_owned(),
                    scaling: ImageBoxImageScaling::Frame(ImageBoxFrame {
                        source: 6.0.into(),
                        destination: 6.0.into(),
                        frame_only: false,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                trigger: ThemedImageMaterial::Image(ImageBoxImage {
                    id: "resources/images/button-trigger.png".to_owned(),
                    scaling: ImageBoxImageScaling::Frame(ImageBoxFrame {
                        source: 6.0.into(),
                        destination: 6.0.into(),
                        frame_only: false,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            },
        )
        .text_variant(
            "title",
            ThemedTextMaterial {
                font: TextBoxFont {
                    name: "resources/fonts/MiKrollFantasy.ttf".to_owned(),
                    size: 64.0,
                },
                ..Default::default()
            },
        )
        .text_variant(
            "option-label",
            ThemedTextMaterial {
                font: TextBoxFont {
                    name: "resources/fonts/MiKrollFantasy.ttf".to_owned(),
                    size: 48.0,
                },
                ..Default::default()
            },
        )
        .text_variant(
            "option-slider",
            ThemedTextMaterial {
                font: TextBoxFont {
                    name: "resources/fonts/MiKrollFantasy.ttf".to_owned(),
                    size: 32.0,
                },
                horizontal_align: TextBoxHorizontalAlign::Center,
                vertical_align: TextBoxVerticalAlign::Middle,
                ..Default::default()
            },
        )
        .text_variant(
            "tab-label",
            ThemedTextMaterial {
                font: TextBoxFont {
                    name: "resources/fonts/MiKrollFantasy.ttf".to_owned(),
                    size: 48.0,
                },
                horizontal_align: TextBoxHorizontalAlign::Center,
                vertical_align: TextBoxVerticalAlign::Middle,
                ..Default::default()
            },
        )
        .text_variant(
            "task-name",
            ThemedTextMaterial {
                font: TextBoxFont {
                    name: "resources/fonts/MiKrollFantasy.ttf".to_owned(),
                    size: 48.0,
                },
                horizontal_align: TextBoxHorizontalAlign::Center,
                vertical_align: TextBoxVerticalAlign::Middle,
                ..Default::default()
            },
        )
        .text_variant(
            "inventory-item-count",
            ThemedTextMaterial {
                font: TextBoxFont {
                    name: "resources/fonts/MiKrollFantasy.ttf".to_owned(),
                    size: 28.0,
                },
                horizontal_align: TextBoxHorizontalAlign::Right,
                vertical_align: TextBoxVerticalAlign::Bottom,
                ..Default::default()
            },
        )
        .switch_variant(
            "checkbox",
            ThemedSwitchMaterial {
                on: ThemedImageMaterial::Image(ImageBoxImage {
                    id: "resources/icons/checkbox-on.png".to_owned(),
                    ..Default::default()
                }),
                off: ThemedImageMaterial::Image(ImageBoxImage {
                    id: "resources/icons/checkbox-off.png".to_owned(),
                    ..Default::default()
                }),
            },
        )
        .slider_variant(
            "",
            ThemedSliderMaterial {
                background: ThemedImageMaterial::Image(ImageBoxImage {
                    id: "resources/images/slider-background.png".to_owned(),
                    scaling: ImageBoxImageScaling::Frame(ImageBoxFrame {
                        source: 6.0.into(),
                        destination: 6.0.into(),
                        frame_only: false,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                filling: ThemedImageMaterial::Image(ImageBoxImage {
                    id: "resources/images/slider-filling.png".to_owned(),
                    scaling: ImageBoxImageScaling::Frame(ImageBoxFrame {
                        source: 6.0.into(),
                        destination: 6.0.into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            },
        )
}
