use raui::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    Light,
    #[default]
    Dark,
}

impl ThemeMode {
    pub fn toggle(&mut self) {
        *self = match *self {
            Self::Dark => Self::Light,
            Self::Light => Self::Dark,
        }
    }
}

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
pub struct TaskProps {
    #[serde(default)]
    pub done: bool,
    #[serde(default)]
    pub name: String,
}

impl TaskProps {
    pub fn new(name: impl ToString) -> Self {
        Self {
            done: false,
            name: name.to_string(),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct AppStateSave {
    theme: ThemeMode,
    tasks: Vec<TaskProps>,
}

pub struct AppState {
    theme: ViewModelValue<ThemeMode>,
    tasks: ViewModelValue<Vec<TaskProps>>,
}

impl AppState {
    pub const VIEW_MODEL: &str = "app-state";
    pub const PROP_THEME: &str = "theme";
    pub const PROP_TASKS: &str = "tasks";

    pub fn new(properties: &mut ViewModelProperties) -> Self {
        Self {
            theme: ViewModelValue::new(ThemeMode::Dark, properties.notifier(Self::PROP_THEME)),
            tasks: ViewModelValue::new(Default::default(), properties.notifier(Self::PROP_TASKS)),
        }
    }

    pub fn theme(&self) -> ThemeMode {
        *self.theme
    }

    pub fn tasks(&self) -> impl Iterator<Item = &TaskProps> {
        self.tasks.iter()
    }

    pub fn toggle_theme(&mut self) {
        self.theme.toggle();
    }

    pub fn add_task(&mut self, name: impl ToString) {
        self.tasks.push(TaskProps::new(name));
    }

    pub fn delete_task(&mut self, index: usize) {
        if index < self.tasks.len() {
            self.tasks.remove(index);
        }
    }

    pub fn toggle_task(&mut self, index: usize) {
        if let Some(task) = self.tasks.get_mut(index) {
            task.done = !task.done;
        }
    }

    pub fn load(&mut self) {
        if let Ok(content) = std::fs::read_to_string("./state.json") {
            if let Ok(state) = serde_json::from_str::<AppStateSave>(&content) {
                *self.theme = state.theme;
                *self.tasks = state.tasks;
            }
        }
    }

    pub fn save(&self) {
        let state = AppStateSave {
            theme: self.theme.to_owned(),
            tasks: self.tasks.iter().cloned().collect(),
        };
        if let Ok(content) = serde_json::to_string_pretty(&state) {
            let _ = std::fs::write("./state.json", content);
        }
    }
}
