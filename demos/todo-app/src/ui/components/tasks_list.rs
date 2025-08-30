use crate::{
    model::{AppState, TaskProps},
    ui::components::confirm_box::{ConfirmBoxProps, ConfirmNotifyMessage, confirm_box},
};
use raui::{
    core::{
        Prefab, PropsData, make_widget, pre_hooks,
        props::PropsData,
        widget::{
            component::{
                containers::{
                    hidden_box::{HiddenBoxProps, hidden_box},
                    horizontal_box::HorizontalBoxProps,
                    vertical_box::{VerticalBoxProps, vertical_box},
                },
                interactive::{
                    button::{ButtonNotifyMessage, ButtonNotifyProps},
                    navigation::{NavContainerActive, NavItemActive},
                    scroll_view::ScrollViewRange,
                },
            },
            context::WidgetContext,
            node::WidgetNode,
            unit::{
                content::ContentBoxItemLayout, flex::FlexBoxItemLayout, text::TextBoxSizeValue,
            },
        },
    },
    material::{
        component::{
            containers::{
                horizontal_paper::horizontal_paper,
                paper::PaperContentLayoutProps,
                scroll_paper::{scroll_paper, scroll_paper_side_scrollbars},
            },
            icon_paper::{IconImage, IconPaperProps},
            interactive::{
                icon_button_paper::icon_button_paper, switch_button_paper::switch_button_paper,
            },
            switch_paper::SwitchPaperProps,
            text_paper::{TextPaperProps, text_paper},
        },
        theme::{ThemeColor, ThemeVariant, ThemedWidgetProps},
    },
};
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Copy, Clone, Serialize, Deserialize)]
struct TaskState {
    #[serde(default)]
    deleting: bool,
}

fn use_task(context: &mut WidgetContext) {
    context.life_cycle.change(|mut context| {
        let mut app_state = context
            .view_models
            .view_model_mut(AppState::VIEW_MODEL)
            .unwrap()
            .write::<AppState>()
            .unwrap();

        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<ButtonNotifyMessage>() {
                if msg.trigger_start() {
                    match msg.sender.key() {
                        "checkbox" => {
                            if let Ok(index) = context.id.key().parse::<usize>() {
                                app_state.toggle_task(index);
                            }
                        }
                        "delete" => {
                            let _ = context.state.write_with(TaskState { deleting: true });
                        }
                        _ => {}
                    }
                }
            } else if let Some(msg) = msg.as_any().downcast_ref::<ConfirmNotifyMessage>() {
                let _ = context.state.write_with(TaskState { deleting: false });
                if msg.confirmed
                    && let Ok(index) = context.id.key().parse::<usize>()
                {
                    app_state.delete_task(index);
                }
            }
        }
    });
}

#[pre_hooks(use_task)]
pub fn task(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        key,
        props,
        state,
        ..
    } = context;
    let data = props.read_cloned_or_default::<TaskProps>();
    let TaskState { deleting } = state.read_cloned_or_default();

    make_widget!(horizontal_paper)
        .key(key)
        .with_props(HorizontalBoxProps {
            separation: 10.0,
            ..Default::default()
        })
        .with_props(ContentBoxItemLayout {
            margin: 10.0.into(),
            ..Default::default()
        })
        .listed_slot(
            make_widget!(switch_button_paper)
                .key("checkbox")
                .with_props(FlexBoxItemLayout {
                    fill: 0.0,
                    grow: 0.0,
                    shrink: 0.0,
                    align: 0.5,
                    ..Default::default()
                })
                .with_props(SwitchPaperProps {
                    on: data.done,
                    variant: "checkbox".to_owned(),
                    size_level: 2,
                })
                .with_props(NavItemActive)
                .with_props(ButtonNotifyProps(id.to_owned().into()))
                .with_props(ThemedWidgetProps {
                    color: ThemeColor::Primary,
                    variant: ThemeVariant::ContentOnly,
                    ..Default::default()
                }),
        )
        .listed_slot(
            make_widget!(text_paper)
                .key("name")
                .with_props(TextPaperProps {
                    text: data.name,
                    height: TextBoxSizeValue::Exact(24.0),
                    variant: "title".to_owned(),
                    ..Default::default()
                })
                .with_props(FlexBoxItemLayout {
                    align: 0.5,
                    ..Default::default()
                }),
        )
        .listed_slot(
            make_widget!(icon_button_paper)
                .key("delete")
                .with_props(FlexBoxItemLayout {
                    fill: 0.0,
                    grow: 0.0,
                    shrink: 0.0,
                    align: 0.5,
                    ..Default::default()
                })
                .with_props(IconPaperProps {
                    image: IconImage {
                        id: "resources/icons/delete.png".to_owned(),
                        ..Default::default()
                    },
                    size_level: 2,
                    ..Default::default()
                })
                .with_props(NavItemActive)
                .with_props(ButtonNotifyProps(id.to_owned().into()))
                .with_props(ThemedWidgetProps {
                    color: ThemeColor::Primary,
                    variant: ThemeVariant::ContentOnly,
                    ..Default::default()
                }),
        )
        .listed_slot(
            make_widget!(hidden_box)
                .with_props(HiddenBoxProps(!deleting))
                .named_slot(
                    "content",
                    make_widget!(confirm_box)
                        .key("confirm")
                        .with_props(ConfirmBoxProps {
                            text: "Do you want to remove task?".to_owned(),
                            notify: id.to_owned().into(),
                        }),
                ),
        )
        .into()
}

fn use_tasks_list(context: &mut WidgetContext) {
    context.life_cycle.mount(|mut context| {
        context
            .view_models
            .bindings(AppState::VIEW_MODEL, AppState::TASKS)
            .unwrap()
            .bind(context.id.to_owned());
    });
}

#[pre_hooks(use_tasks_list)]
pub fn tasks_list(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key, view_models, ..
    } = context;
    let app_state = view_models
        .view_model(AppState::VIEW_MODEL)
        .unwrap()
        .read::<AppState>()
        .unwrap();
    let mut tasks = app_state
        .tasks()
        .enumerate()
        .map(|(index, item)| {
            make_widget!(task)
                .key(index)
                .with_props(item.to_owned())
                .with_props(FlexBoxItemLayout {
                    grow: 0.0,
                    shrink: 0.0,
                    ..Default::default()
                })
        })
        .collect::<Vec<_>>();
    tasks.reverse();

    make_widget!(scroll_paper)
        .key(key)
        .with_props(NavContainerActive)
        .with_props(NavItemActive)
        .with_props(ScrollViewRange::default())
        .with_props(PaperContentLayoutProps(ContentBoxItemLayout {
            margin: 10.0.into(),
            ..Default::default()
        }))
        .named_slot(
            "content",
            make_widget!(vertical_box)
                .key("list")
                .with_props(VerticalBoxProps {
                    separation: 30.0,
                    ..Default::default()
                })
                .listed_slots(tasks),
        )
        .named_slot(
            "scrollbars",
            make_widget!(scroll_paper_side_scrollbars).key("scrollbars"),
        )
        .into()
}
