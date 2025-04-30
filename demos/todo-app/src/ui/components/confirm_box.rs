use raui::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
pub struct ConfirmBoxProps {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub notify: WidgetIdOrRef,
}

#[derive(MessageData, Debug, Clone)]
pub struct ConfirmNotifyMessage {
    #[allow(dead_code)]
    pub sender: WidgetId,
    pub confirmed: bool,
}

fn use_confirm_box(context: &mut WidgetContext) {
    let notify = context
        .props
        .map_or_default::<ConfirmBoxProps, _, _>(|p| p.notify.to_owned());
    let notify = match notify.read() {
        Some(id) => id,
        None => return,
    };

    context.life_cycle.change(move |context| {
        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<ButtonNotifyMessage>() {
                if msg.trigger_start() {
                    match msg.sender.key() {
                        "yes" => {
                            context.messenger.write(
                                notify.to_owned(),
                                ConfirmNotifyMessage {
                                    sender: context.id.to_owned(),
                                    confirmed: true,
                                },
                            );
                        }
                        "no" => {
                            context.messenger.write(
                                notify.to_owned(),
                                ConfirmNotifyMessage {
                                    sender: context.id.to_owned(),
                                    confirmed: false,
                                },
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
    });
}

#[pre_hooks(use_confirm_box)]
pub fn confirm_box(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext { id, key, props, .. } = context;
    let ConfirmBoxProps { text, .. } = props.read_cloned_or_default();

    make_widget!(modal_paper)
        .key(key)
        .named_slot(
            "content",
            make_widget!(vertical_paper)
                .key("list")
                .with_props(ContentBoxItemLayout {
                    anchors: 0.5.into(),
                    margin: Rect {
                        left: -200.0,
                        right: -200.0,
                        top: -100.0,
                        bottom: -100.0,
                    },
                    ..Default::default()
                })
                .with_props(VerticalBoxProps {
                    separation: 20.0,
                    ..Default::default()
                })
                .listed_slot(
                    make_widget!(wrap_box)
                        .key("text-wrap")
                        .with_props(WrapBoxProps {
                            margin: 16.0.into(),
                            ..Default::default()
                        })
                        .named_slot(
                            "content",
                            make_widget!(text_paper)
                                .key("text")
                                .with_props(TextPaperProps {
                                    text,
                                    height: TextBoxSizeValue::Exact(24.0),
                                    variant: "title".to_owned(),
                                    ..Default::default()
                                }),
                        ),
                )
                .listed_slot(
                    make_widget!(horizontal_box)
                        .key("buttons")
                        .listed_slot(
                            make_widget!(wrap_box)
                                .key("yes-wrap")
                                .with_props(WrapBoxProps {
                                    margin: 16.0.into(),
                                    ..Default::default()
                                })
                                .named_slot(
                                    "content",
                                    make_widget!(text_button_paper)
                                        .key("yes")
                                        .with_props(TextPaperProps {
                                            text: "YES".to_owned(),
                                            height: TextBoxSizeValue::Exact(24.0),
                                            variant: "button".to_owned(),
                                            ..Default::default()
                                        })
                                        .with_props(WrapBoxProps {
                                            margin: 16.0.into(),
                                            ..Default::default()
                                        })
                                        .with_props(NavItemActive)
                                        .with_props(ButtonNotifyProps(id.to_owned().into())),
                                ),
                        )
                        .listed_slot(
                            make_widget!(wrap_box)
                                .key("no-wrap")
                                .with_props(WrapBoxProps {
                                    margin: 16.0.into(),
                                    ..Default::default()
                                })
                                .named_slot(
                                    "content",
                                    make_widget!(text_button_paper)
                                        .key("no")
                                        .with_props(TextPaperProps {
                                            text: "NO".to_owned(),
                                            height: TextBoxSizeValue::Exact(24.0),
                                            variant: "button".to_owned(),
                                            ..Default::default()
                                        })
                                        .with_props(WrapBoxProps {
                                            margin: 16.0.into(),
                                            ..Default::default()
                                        })
                                        .with_props(NavItemActive)
                                        .with_props(ButtonNotifyProps(id.to_owned().into())),
                                ),
                        ),
                ),
        )
        .into()
}
