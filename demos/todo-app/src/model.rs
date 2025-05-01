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
    creating_task: ViewModelValue<bool>,
    new_task_name: Managed<ViewModelValue<String>>,
}

impl AppState {
    pub const VIEW_MODEL: &str = "app-state";
    pub const THEME: &str = "theme";
    pub const TASKS: &str = "tasks";
    pub const CREATING_TASK: &str = "creating-task";
    pub const NEW_TASK_NAME: &str = "new-task-name";

    pub fn new(properties: &mut ViewModelProperties) -> Self {
        Self {
            theme: ViewModelValue::new(ThemeMode::Dark, properties.notifier(Self::THEME)),
            tasks: ViewModelValue::new(Default::default(), properties.notifier(Self::TASKS)),
            creating_task: ViewModelValue::new(false, properties.notifier(Self::CREATING_TASK)),
            new_task_name: Managed::new(ViewModelValue::new(
                Default::default(),
                properties.notifier(Self::NEW_TASK_NAME),
            )),
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

    pub fn creating_task(&self) -> bool {
        *self.creating_task
    }

    pub fn new_task_name(&mut self) -> ManagedLazy<ViewModelValue<String>> {
        self.new_task_name.lazy()
    }

    pub fn create_task(&mut self) {
        *self.creating_task = true;
        **self.new_task_name.write().unwrap() = Default::default();
    }

    pub fn add_task(&mut self) {
        if *self.creating_task {
            *self.creating_task = false;
            let name = std::mem::take(&mut **self.new_task_name.write().unwrap());
            if !name.is_empty() {
                self.tasks.push(TaskProps::new(name));
            }
        }
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
