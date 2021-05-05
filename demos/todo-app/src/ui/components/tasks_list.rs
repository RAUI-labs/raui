use crate::ui::components::{
    app::{AppMessage, AppSharedProps},
    confirm_box::{confirm_box, ConfirmBoxProps, ConfirmNotifyMessage},
};
use raui_core::prelude::*;
use raui_material::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
pub struct TaskProps {
    #[serde(default)]
    pub done: bool,
    #[serde(default)]
    pub name: String,
}

impl TaskProps {
    pub fn new(name: &str) -> Self {
        Self {
            done: false,
            name: name.to_owned(),
        }
    }
}

#[derive(PropsData, Debug, Default, Copy, Clone, Serialize, Deserialize)]
struct TaskState {
    #[serde(default)]
    deleting: bool,
}

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
pub struct TasksProps {
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tasks: Vec<TaskProps>,
}

fn use_task(context: &mut WidgetContext) {
    context.life_cycle.change(|context| {
        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<ButtonNotifyMessage>() {
                if msg.trigger_start() {
                    match msg.sender.key() {
                        "checkbox" => {
                            // TODO: figure out better to pass index to the message.
                            // maybe using props? anything would be better than parsing string.
                            if let Ok(index) = context.id.key().parse::<usize>() {
                                let id = context
                                    .shared_props
                                    .read_cloned_or_default::<AppSharedProps>()
                                    .id;
                                context.messenger.write(id, AppMessage::ToggleTask(index));
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
                if msg.confirmed {
                    // TODO: figure out better to pass index to the message.
                    // maybe using props? anything would be better than parsing string.
                    if let Ok(index) = context.id.key().parse::<usize>() {
                        let id = context
                            .shared_props
                            .read_cloned_or_default::<AppSharedProps>()
                            .id;
                        context.messenger.write(id, AppMessage::DeleteTask(index));
                    }
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

    let checkbox_props = Props::new(FlexBoxItemLayout {
        fill: 0.0,
        grow: 0.0,
        shrink: 0.0,
        align: 0.5,
        ..Default::default()
    })
    .with(SwitchPaperProps {
        on: data.done,
        variant: "checkbox".to_owned(),
        size_level: 2,
    })
    .with(NavItemActive)
    .with(ButtonNotifyProps(id.to_owned().into()))
    .with(ThemedWidgetProps {
        color: ThemeColor::Primary,
        variant: ThemeVariant::ContentOnly,
    });

    let name_props = Props::new(TextPaperProps {
        text: data.name,
        height: TextBoxSizeValue::Exact(24.0),
        variant: "title".to_owned(),
        ..Default::default()
    })
    .with(FlexBoxItemLayout {
        align: 0.5,
        ..Default::default()
    });

    let delete_props = Props::new(FlexBoxItemLayout {
        fill: 0.0,
        grow: 0.0,
        shrink: 0.0,
        align: 0.5,
        ..Default::default()
    })
    .with(IconPaperProps {
        image: IconImage {
            id: "icon-delete".to_owned(),
            ..Default::default()
        },
        size_level: 2,
        ..Default::default()
    })
    .with(NavItemActive)
    .with(ButtonNotifyProps(id.to_owned().into()))
    .with(ThemedWidgetProps {
        color: ThemeColor::Primary,
        variant: ThemeVariant::ContentOnly,
    });

    let list_props = Props::new(HorizontalBoxProps {
        separation: 10.0,
        ..Default::default()
    })
    .with(ContentBoxItemLayout {
        margin: Rect {
            left: 10.0,
            right: 0.0,
            top: 10.0,
            bottom: 10.0,
        },
        ..Default::default()
    });

    let confirm_props = ConfirmBoxProps {
        text: "Do you want to remove task?".to_owned(),
        notify: id.to_owned().into(),
    };

    widget! {
        (#{key} horizontal_paper: {list_props} [
            (#{"checkbox"} switch_button_paper: {checkbox_props})
            (#{"name"} text_paper: {name_props})
            (#{"delete"} icon_button_paper: {delete_props})
            (hidden_box: {HiddenBoxProps(!deleting)} {
                content = (#{"confirm"} confirm_box: {confirm_props})
            })
        ])
    }
}

pub fn tasks_list(context: WidgetContext) -> WidgetNode {
    let WidgetContext { id, key, props, .. } = context;

    let TasksProps { tasks } = props.read_cloned_or_default();

    let tasks = tasks
        .into_iter()
        .enumerate()
        .map(|(i, item)| {
            let props = Props::new(item).with(FlexBoxItemLayout {
                grow: 0.0,
                shrink: 0.0,
                ..Default::default()
            });
            widget! { (#{i} task: {props}) }
        })
        .collect::<Vec<_>>();

    let scroll_props = Props::new(NavContainerActive)
        .with(NavItemActive)
        .with(ScrollViewNotifyProps(id.to_owned().into()))
        .with(ScrollViewRange::default());

    let list_props = VerticalBoxProps {
        separation: 10.0,
        ..Default::default()
    };

    widget! {
        (#{key} nav_scroll_paper: {scroll_props} {
            content = (#{"list"} vertical_box: {list_props} |[ tasks ]|)
            scrollbars = (#{"scrollbars"} nav_scroll_paper_side_scrollbars)
        })
    }
}
