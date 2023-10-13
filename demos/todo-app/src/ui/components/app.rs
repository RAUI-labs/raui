use crate::{
    app::AppSignal,
    ui::components::{
        app_bar::app_bar,
        tasks_list::{tasks_list, TaskProps, TasksProps},
    },
};
use raui_core::prelude::*;
use raui_material::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    Light,
    #[default]
    Dark,
}

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
pub struct AppState {
    #[serde(default)]
    pub theme: ThemeMode,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tasks: Vec<TaskProps>,
}

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
pub struct AppSharedProps {
    #[serde(default)]
    pub id: WidgetId,
}

#[derive(MessageData, Debug, Clone)]
pub enum AppMessage {
    ToggleTheme,
    AddTask(String),
    DeleteTask(usize),
    ToggleTask(usize),
    Save,
    Load(AppState),
}

fn new_theme(theme: ThemeMode) -> ThemeProps {
    let mut theme = match theme {
        ThemeMode::Light => new_light_theme(),
        ThemeMode::Dark => new_dark_theme(),
    };
    theme.text_variants.insert(
        "title".to_owned(),
        ThemedTextMaterial {
            font: TextBoxFont {
                name: "bold".to_owned(),
                size: 24.0,
            },
            ..Default::default()
        },
    );
    theme.switch_variants.insert(
        "checkbox".to_owned(),
        ThemedSwitchMaterial {
            on: ThemedImageMaterial::Image(ImageBoxImage {
                id: "icon-check-box-on".to_owned(),
                ..Default::default()
            }),
            off: ThemedImageMaterial::Image(ImageBoxImage {
                id: "icon-check-box-off".to_owned(),
                ..Default::default()
            }),
        },
    );
    theme
}

fn use_app(context: &mut WidgetContext) {
    context.life_cycle.mount(|context| {
        let _ = context.state.write(AppState::default());
        context
            .signals
            .write(AppSignal::Ready(context.id.to_owned()));
    });

    context.life_cycle.change(|context| {
        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref() {
                match msg {
                    AppMessage::ToggleTheme => {
                        let mut data = match context.state.read::<AppState>() {
                            Ok(state) => state.clone(),
                            Err(_) => AppState::default(),
                        };
                        data.theme = match data.theme {
                            ThemeMode::Light => ThemeMode::Dark,
                            ThemeMode::Dark => ThemeMode::Light,
                        };
                        let _ = context.state.write(data);
                    }
                    AppMessage::AddTask(name) => {
                        let mut data = match context.state.read::<AppState>() {
                            Ok(state) => state.clone(),
                            Err(_) => AppState::default(),
                        };
                        data.tasks.push(TaskProps::new(name));
                        let _ = context.state.write(data);
                    }
                    AppMessage::DeleteTask(index) => {
                        let mut data = match context.state.read::<AppState>() {
                            Ok(state) => state.clone(),
                            Err(_) => AppState::default(),
                        };
                        data.tasks.remove(*index);
                        let _ = context.state.write(data);
                    }
                    AppMessage::ToggleTask(index) => {
                        let mut data = match context.state.read::<AppState>() {
                            Ok(state) => state.clone(),
                            Err(_) => AppState::default(),
                        };
                        if let Some(item) = data.tasks.get_mut(*index) {
                            item.done = !item.done;
                        }
                        let _ = context.state.write(data);
                    }
                    AppMessage::Save => {
                        if let Ok(data) = context.state.read::<AppState>() {
                            context.signals.write(AppSignal::Save(data.clone()));
                        }
                    }
                    AppMessage::Load(data) => {
                        let _ = context.state.write(data.clone());
                    }
                }
            }
        }
    });
}

#[pre_hooks(use_nav_container_active, use_app)]
pub fn app(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext { id, key, state, .. } = context;

    let (theme_mode, tasks) =
        state.map_or_default::<AppState, _, _>(|s| (s.theme, s.tasks.clone()));
    let theme = new_theme(theme_mode);
    let idref = WidgetRef::default();

    let shared_props = Props::new(AppSharedProps { id: id.to_owned() })
        .with(PortalsContainer(idref.clone()))
        .with(theme)
        .with(theme_mode);

    let bar_props = FlexBoxItemLayout {
        grow: 0.0,
        shrink: 0.0,
        ..Default::default()
    };

    let wrap_props = WrapBoxProps {
        margin: Rect {
            left: 32.0,
            right: 32.0,
            top: 32.0,
            bottom: 32.0,
        },
        fill: true,
    };

    let list_props = VerticalBoxProps {
        separation: 10.0,
        ..Default::default()
    };

    widget! {
        (#{key} | {idref} paper | {shared_props} [
            (#{"wrap"} wrap_box: {wrap_props} {
                content = (#{"list"} vertical_box: {list_props} [
                    (#{"app-bar"} app_bar: {bar_props})
                    (#{"tasks-list"} tasks_list: {TasksProps { tasks }})
                ])
            })
        ])
    }
}
