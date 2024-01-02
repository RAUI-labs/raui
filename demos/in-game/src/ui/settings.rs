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

    make_widget!(window_paper)
        .key("settings")
        .with_props(WindowPaperProps {
            bar_margin: 20.0.into(),
            bar_height: Some(80.0),
            content_margin: 40.0.into(),
            ..Default::default()
        })
        .named_slot(
            "bar",
            make_widget!(text_paper)
                .key("title")
                .with_props(TextPaperProps {
                    text: "SETTINGS".to_owned(),
                    variant: "title".to_owned(),
                    use_main_color: true,
                    ..Default::default()
                }),
        )
        .named_slot(
            "content",
            make_widget!(nav_vertical_box)
                .key("settings")
                .with_props(VerticalBoxProps {
                    override_slots_layout: Some(FlexBoxItemLayout {
                        grow: 0.0,
                        shrink: 0.0,
                        basis: Some(48.0),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .listed_slot(
                    make_widget!(option)
                        .key("fullscreen")
                        .with_props("Fullscreen".to_owned())
                        .named_slot(
                            "content",
                            make_widget!(switch_button_paper)
                                .key("button?id=fullscreen")
                                .with_props(NavItemActive)
                                .with_props(ButtonNotifyProps(context.id.to_owned().into()))
                                .with_props(SwitchPaperProps {
                                    on: *settings.fullscreen,
                                    variant: "checkbox".to_owned(),
                                    size_level: 3,
                                })
                                .with_props(ThemedWidgetProps {
                                    color: ThemeColor::Primary,
                                    variant: ThemeVariant::ContentOnly,
                                    ..Default::default()
                                }),
                        ),
                )
                .listed_slot(
                    make_widget!(option)
                        .key("volume")
                        .with_props("Volume".to_owned())
                        .named_slot(
                            "content",
                            make_widget!(numeric_slider_paper)
                                .key("slider")
                                .with_props(NavItemActive)
                                .with_props(TextPaperProps {
                                    variant: "option-slider".to_owned(),
                                    use_main_color: true,
                                    ..Default::default()
                                })
                                .with_props(SliderViewProps {
                                    input: Some(SliderInput::new(settings.volume.lazy())),
                                    from: 0.0,
                                    to: 100.0,
                                    direction: SliderViewDirection::LeftToRight,
                                })
                                .with_props(NumericSliderPaperProps {
                                    fractional_digits_count: Some(0),
                                }),
                        ),
                ),
        )
        .into()
}

fn use_settings(context: &mut WidgetContext) {
    context.life_cycle.change(|mut context| {
        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<ButtonNotifyMessage>() {
                if msg.trigger_start() {
                    if let Some(id) = WidgetIdMetaParams::new(msg.sender.meta()).find_value("id") {
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
        }
    });
}

fn option(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        props,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => { content });
    let label = props.read_cloned_or_default::<String>();

    make_widget!(horizontal_box)
        .key(key)
        .with_props(HorizontalBoxProps {
            separation: 50.0,
            ..Default::default()
        })
        .listed_slot(
            make_widget!(text_paper)
                .key("label")
                .with_props(TextPaperProps {
                    text: label,
                    variant: "option-label".to_owned(),
                    use_main_color: true,
                    horizontal_align_override: Some(TextBoxHorizontalAlign::Right),
                    ..Default::default()
                }),
        )
        .listed_slot(content)
        .into()
}
