use crate::model::{AppState, ThemeMode};
use raui::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
pub struct AppBarState {
    pub creating_task: bool,
    pub new_task_name: String,
}

fn use_app_bar(context: &mut WidgetContext) {
    context.life_cycle.mount(|context| {
        let _ = context.state.write(AppBarState::default());
    });

    context.life_cycle.change(|mut context| {
        let mut app_state = context
            .view_models
            .view_model_mut::<AppState>(AppState::VIEW_MODEL)
            .unwrap();

        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<ButtonNotifyMessage>() {
                if msg.trigger_start() {
                    match msg.sender.key() {
                        "theme" => {
                            app_state.toggle_theme();
                        }
                        "save" => {
                            app_state.save();
                        }
                        "create" => {
                            let _ = context.state.write(AppBarState {
                                creating_task: true,
                                ..Default::default()
                            });
                        }
                        "add" => {
                            if let Ok(data) = context.state.read::<AppBarState>() {
                                if !data.new_task_name.is_empty() {
                                    app_state.add_task(data.new_task_name.to_owned());
                                }
                            }
                            let _ = context.state.write(AppBarState::default());
                        }
                        _ => {}
                    }
                }
            } else if let Some(msg) = msg.as_any().downcast_ref::<TextInputNotifyMessage>() {
                let _ = context.state.write(AppBarState {
                    creating_task: true,
                    new_task_name: msg.state.text.to_owned(),
                });
            }
        }
    });
}

#[pre_hooks(use_app_bar)]
pub fn app_bar(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        id,
        state,
        view_models,
        ..
    } = context;
    let app_state = view_models
        .view_model::<AppState>(AppState::VIEW_MODEL)
        .unwrap();
    let creating_task = state
        .read::<AppBarState>()
        .map(|state| state.creating_task)
        .unwrap_or_default();

    make_widget!(vertical_box)
        .key(key)
        .with_props(VerticalBoxProps {
            separation: 10.0,
            ..Default::default()
        })
        .listed_slot(
            make_widget!(horizontal_box)
                .key("title-bar")
                .with_props(HorizontalBoxProps {
                    separation: 10.0,
                    ..Default::default()
                })
                .listed_slot(
                    make_widget!(text_paper)
                        .key("title")
                        .with_props(TextPaperProps {
                            text: "TODO App".to_owned(),
                            variant: "title".to_owned(),
                            ..Default::default()
                        }),
                )
                .listed_slot(
                    make_widget!(text_tooltip_paper)
                        .merge_props(make_tooltip_props("Change theme"))
                        .named_slot(
                            "content",
                            make_widget!(icon_button_paper).key("theme").merge_props(
                                make_icon_props(
                                    id,
                                    if app_state.theme() == ThemeMode::Dark {
                                        "resources/icons/light-mode.png"
                                    } else {
                                        "resources/icons/dark-mode.png"
                                    },
                                ),
                            ),
                        ),
                )
                .listed_slot(
                    make_widget!(text_tooltip_paper)
                        .merge_props(make_tooltip_props("Save changes"))
                        .named_slot(
                            "content",
                            make_widget!(icon_button_paper)
                                .key("save")
                                .merge_props(make_icon_props(id, "resources/icons/save.png")),
                        ),
                )
                .listed_slot(if creating_task {
                    WidgetNode::default()
                } else {
                    make_widget!(text_tooltip_paper)
                        .merge_props(make_tooltip_props("Create task"))
                        .named_slot(
                            "content",
                            make_widget!(icon_button_paper)
                                .key("create")
                                .merge_props(make_icon_props(id, "resources/icons/add.png")),
                        )
                        .into()
                }),
        )
        .listed_slot(if creating_task {
            make_widget!(horizontal_box)
                .key("task-bar")
                .with_props(HorizontalBoxProps {
                    separation: 10.0,
                    ..Default::default()
                })
                .listed_slot(
                    make_widget!(text_field_paper)
                        .key("name")
                        .with_props(TextFieldPaperProps {
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
                            variant: "input".to_owned(),
                            ..Default::default()
                        })
                        .with_props(NavItemActive)
                        .with_props(ButtonNotifyProps(id.to_owned().into()))
                        .with_props(TextInputNotifyProps(id.to_owned().into())),
                )
                .listed_slot(
                    make_widget!(text_tooltip_paper)
                        .merge_props(make_tooltip_props("Confirm new task"))
                        .named_slot(
                            "content",
                            make_widget!(icon_button_paper)
                                .key("add")
                                .merge_props(make_icon_props(id, "resources/icons/add.png")),
                        ),
                )
                .into()
        } else {
            WidgetNode::default()
        })
        .into()
}

fn make_tooltip_props(hint: &str) -> Props {
    Props::new(FlexBoxItemLayout {
        fill: 0.0,
        grow: 0.0,
        shrink: 0.0,
        align: 0.5,
        ..Default::default()
    })
    .with(PivotBoxProps {
        pivot: Vec2 { x: 1.0, y: 1.0 },
        align: Vec2 { x: 1.0, y: 0.0 },
    })
    .with(TextPaperProps {
        text: hint.to_owned(),
        width: TextBoxSizeValue::Exact(150.0),
        height: TextBoxSizeValue::Exact(24.0),
        variant: "tooltip".to_owned(),
        ..Default::default()
    })
}

fn make_icon_props(id: &WidgetId, image_id: impl ToString) -> Props {
    Props::new(IconPaperProps {
        image: IconImage {
            id: image_id.to_string(),
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
    .with(ButtonNotifyProps(id.to_owned().into()))
}
