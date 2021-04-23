use crate::ui::components::app::{AppMessage, AppSharedProps, ThemeMode};
use raui_core::prelude::*;
use raui_material::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
pub struct AppBarState {
    pub creating_task: bool,
    pub new_task_name: String,
}

fn use_app_bar(context: &mut WidgetContext) {
    context.life_cycle.mount(|context| {
        drop(context.state.write(AppBarState::default()));
    });

    context.life_cycle.change(|context| {
        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<ButtonNotifyMessage>() {
                if msg.trigger_start() {
                    match msg.sender.key() {
                        "theme" => {
                            let id = context
                                .shared_props
                                .read_cloned_or_default::<AppSharedProps>()
                                .id;
                            context.messenger.write(id, AppMessage::ToggleTheme);
                        }
                        "save" => {
                            let id = context
                                .shared_props
                                .read_cloned_or_default::<AppSharedProps>()
                                .id;
                            context.messenger.write(id, AppMessage::Save);
                        }
                        "create" => {
                            drop(context.state.write(AppBarState {
                                creating_task: true,
                                ..Default::default()
                            }));
                        }
                        "add" => {
                            if let Ok(data) = context.state.read::<AppBarState>() {
                                if !data.new_task_name.is_empty() {
                                    let id = context
                                        .shared_props
                                        .read_cloned_or_default::<AppSharedProps>()
                                        .id;
                                    context.messenger.write(
                                        id,
                                        AppMessage::AddTask(data.new_task_name.to_owned()),
                                    );
                                }
                            }
                            drop(context.state.write(AppBarState::default()));
                        }
                        _ => {}
                    }
                }
            } else if let Some(msg) = msg.as_any().downcast_ref::<TextInputNotifyMessage>() {
                drop(context.state.write(AppBarState {
                    creating_task: true,
                    new_task_name: msg.state.text.to_owned(),
                }));
            }
        }
    });
}

#[pre_hooks(use_app_bar)]
pub fn app_bar(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        props,
        shared_props,
        state,
        ..
    } = context;

    let theme_mode = shared_props.read_cloned_or_default::<ThemeMode>();
    let props = props.clone().with(VerticalBoxProps {
        separation: 10.0,
        ..Default::default()
    });
    let line_props = props.clone().with(HorizontalBoxProps {
        separation: 10.0,
        ..Default::default()
    });
    let title_props = TextPaperProps {
        text: "TODO Demo App".to_owned(),
        variant: "title".to_owned(),
        ..Default::default()
    };
    let name_props = Props::new(TextFieldPaperProps {
        hint: "> Type new task name...".to_owned(),
        paper_theme: ThemedWidgetProps {
            color: ThemeColor::Primary,
            ..Default::default()
        },
        padding: Rect {
            left: 10.0,
            right: 10.0,
            top: 6.0,
            bottom: 6.0,
        },
        ..Default::default()
    })
    .with(NavItemActive)
    .with(ButtonNotifyProps(id.to_owned().into()))
    .with(TextInputNotifyProps(id.to_owned().into()));
    let theme_props = Props::new(FlexBoxItemLayout {
        fill: 0.0,
        grow: 0.0,
        shrink: 0.0,
        align: 0.5,
        ..Default::default()
    })
    .with(IconPaperProps {
        image: IconImage {
            id: if theme_mode == ThemeMode::Dark {
                "icon-light-mode".to_owned()
            } else {
                "icon-dark-mode".to_owned()
            },
            ..Default::default()
        },
        size_level: 2,
        ..Default::default()
    })
    .with(ThemedWidgetProps {
        color: ThemeColor::Secondary,
        variant: ThemeVariant::ContentOnly,
    })
    .with(NavItemActive)
    .with(ButtonNotifyProps(id.to_owned().into()));
    let save_props = Props::new(FlexBoxItemLayout {
        fill: 0.0,
        grow: 0.0,
        shrink: 0.0,
        align: 0.5,
        ..Default::default()
    })
    .with(IconPaperProps {
        image: IconImage {
            id: "icon-save".to_owned(),
            ..Default::default()
        },
        size_level: 2,
        ..Default::default()
    })
    .with(ThemedWidgetProps {
        color: ThemeColor::Secondary,
        variant: ThemeVariant::ContentOnly,
    })
    .with(NavItemActive)
    .with(ButtonNotifyProps(id.to_owned().into()));
    let create_props = Props::new(FlexBoxItemLayout {
        fill: 0.0,
        grow: 0.0,
        shrink: 0.0,
        align: 0.5,
        ..Default::default()
    })
    .with(IconPaperProps {
        image: IconImage {
            id: "icon-add".to_owned(),
            ..Default::default()
        },
        size_level: 2,
        ..Default::default()
    })
    .with(ThemedWidgetProps {
        color: ThemeColor::Secondary,
        variant: ThemeVariant::ContentOnly,
    })
    .with(NavItemActive)
    .with(ButtonNotifyProps(id.to_owned().into()));
    let creating_task = match state.read::<AppBarState>() {
        Ok(state) => state.creating_task,
        Err(_) => false,
    };

    widget! {
        (#{"content"} vertical_box: {props} [
            (#{"title-bar"} horizontal_box: {line_props.clone()} [
                (#{"title"} text_paper: {title_props})
                (#{"theme"} icon_button_paper: {theme_props})
                (#{"save"} icon_button_paper: {save_props})
                {
                    if creating_task {
                        widget!{()}
                    } else {
                        widget! { (#{"create"} icon_button_paper: {create_props.clone()}) }
                    }
                }
            ])
            {
                if creating_task {
                    widget! {
                        (#{"task-bar"} horizontal_box: {line_props} [
                            (#{"name"} text_field_paper: {name_props})
                            (#{"add"} icon_button_paper: {create_props})
                        ])
                    }
                } else {
                    widget!{()}
                }
            }
        ])
    }
}
