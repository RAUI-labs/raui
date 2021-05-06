use raui_core::prelude::*;
use raui_material::prelude::*;
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

    let list_props = Props::new(ContentBoxItemLayout {
        anchors: Rect {
            left: 0.5,
            right: 0.5,
            top: 0.5,
            bottom: 0.5,
        },
        margin: Rect {
            left: -200.0,
            right: -200.0,
            top: -100.0,
            bottom: -100.0,
        },
        ..Default::default()
    })
    .with(VerticalBoxProps {
        separation: 20.0,
        ..Default::default()
    });

    let wrap_props = WrapBoxProps {
        margin: Rect {
            left: 16.0,
            right: 16.0,
            top: 16.0,
            bottom: 16.0,
        },
        fill: true,
    };

    let text_props = TextPaperProps {
        text,
        height: TextBoxSizeValue::Exact(24.0),
        variant: "title".to_owned(),
        horizontal_align_override: Some(TextBoxHorizontalAlign::Center),
        ..Default::default()
    };

    let yes_props = Props::new(TextPaperProps {
        text: "YES".to_owned(),
        height: TextBoxSizeValue::Exact(24.0),
        variant: "title".to_owned(),
        horizontal_align_override: Some(TextBoxHorizontalAlign::Center),
        ..Default::default()
    })
    .with(wrap_props.clone())
    .with(NavItemActive)
    .with(ButtonNotifyProps(id.to_owned().into()));

    let no_props = Props::new(TextPaperProps {
        text: "NO".to_owned(),
        height: TextBoxSizeValue::Exact(24.0),
        variant: "title".to_owned(),
        horizontal_align_override: Some(TextBoxHorizontalAlign::Center),
        ..Default::default()
    })
    .with(wrap_props.clone())
    .with(NavItemActive)
    .with(ButtonNotifyProps(id.to_owned().into()));

    widget! {
        (#{key} modal_paper {
            content = (#{"list"} vertical_paper: {list_props} [
                (#{"text-wrap"} wrap_box: {wrap_props.clone()} {
                    content = (#{"text"} text_paper: {text_props})
                })
                (#{"buttons"} horizontal_paper [
                    (#{"yes-wrap"} wrap_box: {wrap_props.clone()} {
                        content = (#{"yes"} text_button_paper: {yes_props})
                    })
                    (#{"no-wrap"} wrap_box: {wrap_props} {
                        content = (#{"no"} text_button_paper: {no_props})
                    })
                ])
            ])
        })
    }
}
