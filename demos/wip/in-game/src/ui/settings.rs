use crate::model::settings::Settings;
use raui::prelude::*;

#[pre_hooks(use_settings)]
pub fn settings(mut context: WidgetContext) -> WidgetNode {
    let settings = context
        .view_models
        .view_model(Settings::VIEW_MODEL)
        .unwrap()
        .read::<Settings>()
        .unwrap();
    let idref = WidgetRef::default();

    make_widget!(nav_vertical_box)
        .key("settings")
        .idref(idref.clone())
        .listed_slot(
            make_widget!(text_box)
                .key("title")
                .with_props(TextBoxProps {
                    text: "SETTINGS".to_owned(),
                    horizontal_align: TextBoxHorizontalAlign::Center,
                    vertical_align: TextBoxVerticalAlign::Middle,
                    font: TextBoxFont {
                        name: "resources/fonts/MiKrollFantasy.ttf".to_owned(),
                        size: 64.0,
                    },
                    ..Default::default()
                }),
        )
        .listed_slot(make_fullscreen_toggle(idref, *settings.fullscreen))
        .into()
}

fn make_fullscreen_toggle(idref: WidgetRef, value: bool) -> WidgetNode {
    make_widget!(horizontal_box)
        .key("fullscreen")
        .listed_slot(
            make_widget!(text_box)
                .key("label")
                .with_props(TextBoxProps {
                    text: "Fullscreen".to_owned(),
                    horizontal_align: TextBoxHorizontalAlign::Right,
                    vertical_align: TextBoxVerticalAlign::Middle,
                    font: TextBoxFont {
                        name: "resources/fonts/MiKrollFantasy.ttf".to_owned(),
                        size: 48.0,
                    },
                    ..Default::default()
                }),
        )
        .listed_slot(
            make_widget!(button)
                .key("button?id=fullscreen")
                .with_props(ButtonNotifyProps(idref.into()))
                .named_slot(
                    "content",
                    make_widget!(text_box).key("text").with_props(TextBoxProps {
                        text: if value { "ON" } else { "OFF" }.to_owned(),
                        vertical_align: TextBoxVerticalAlign::Middle,
                        font: TextBoxFont {
                            name: "resources/fonts/MiKrollFantasy.ttf".to_owned(),
                            size: 48.0,
                        },
                        ..Default::default()
                    }),
                ),
        )
        .into()
}

fn use_settings(context: &mut WidgetContext) {
    context.life_cycle.change(|mut context| {
        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<ButtonNotifyMessage>() {
                if let Some(id) = WidgetIdMetaParams::new(msg.sender.key()).find_value("id") {
                    if id == "fullscreen" {
                        let mut settings = context
                            .view_models
                            .view_model_mut(Settings::VIEW_MODEL)
                            .unwrap()
                            .write::<Settings>()
                            .unwrap();
                        *settings.fullscreen = !*settings.fullscreen;
                    }
                }
            }
        }
    });
}
