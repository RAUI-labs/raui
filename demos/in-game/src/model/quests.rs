use raui::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    path::Path,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quest {
    pub name: String,
}

pub struct Quests {
    database: HashMap<String, Quest>,
    completed: ViewModelValue<HashSet<String>>,
}

impl Quests {
    pub const VIEW_MODEL: &str = "quests";
    pub const COMPLETED: &str = "completed";

    pub fn view_model(database_path: impl AsRef<Path>) -> ViewModel {
        let database_path = database_path.as_ref();
        let database = File::open(database_path).unwrap_or_else(|err| {
            panic!(
                "Could not load quests database: {:?}. Error: {}",
                database_path, err
            )
        });
        let database = serde_json::from_reader(database).unwrap_or_else(|err| {
            panic!(
                "Could not deserialize quests database: {:?}. Error: {}",
                database_path, err
            )
        });

        ViewModel::produce(|properties| {
            let mut result = Self {
                database,
                completed: ViewModelValue::new(
                    Default::default(),
                    properties.notifier(Self::COMPLETED),
                ),
            };
            result.toggle("collect-3-potions");
            result
        })
    }

    pub fn toggle(&mut self, id: impl ToString) {
        let id = id.to_string();
        if self.completed.contains(&id) {
            self.completed.remove(&id);
        } else {
            self.completed.insert(id);
        }
    }

    pub fn completed(&self) -> impl Iterator<Item = (&str, &Quest)> {
        let completed = &*self.completed;
        self.database
            .iter()
            .filter(|(id, _)| completed.contains(*id))
            .map(|(id, quest)| (id.as_str(), quest))
    }

    pub fn available(&self) -> impl Iterator<Item = (&str, &Quest)> {
        let completed = &*self.completed;
        self.database
            .iter()
            .filter(|(id, _)| !completed.contains(*id))
            .map(|(id, quest)| (id.as_str(), quest))
    }
}
