use raui::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub icon: String,
    pub buy: usize,
    pub sell: usize,
}

pub struct ItemsDatabase {
    pub items: HashMap<String, Item>,
}

impl ItemsDatabase {
    pub const VIEW_MODEL: &'static str = "items-database";

    pub fn view_model(database_path: impl AsRef<Path>) -> ViewModel {
        let database_path = database_path.as_ref();
        let items = File::open(database_path).unwrap_or_else(|err| {
            panic!(
                "Could not load items database: {:?}. Error: {}",
                database_path, err
            )
        });
        let items = serde_json::from_reader(items).unwrap_or_else(|err| {
            panic!(
                "Could not deserialize items database: {:?}. Error: {}",
                database_path, err
            )
        });

        ViewModel::new_object(Self { items })
    }
}

pub struct Inventory {
    owned: ViewModelValue<HashMap<String, usize>>,
}

impl Inventory {
    pub const VIEW_MODEL: &'static str = "items";
    const OWNED: &'static str = "owned";

    pub fn view_model() -> ViewModel {
        ViewModel::produce(|properties| {
            let mut result = Self {
                owned: ViewModelValue::new(Default::default(), properties.notifier(Self::OWNED)),
            };
            result.add("potion", 5);
            result.add("shield", 1);
            result.add("sword", 2);
            result
        })
    }

    pub fn add(&mut self, id: impl ToString, count: usize) {
        let value = self.owned.entry(id.to_string()).or_default();
        *value = value.saturating_add(count);
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, id: &str, count: usize) {
        if let Some(value) = self.owned.get_mut(id) {
            *value = value.saturating_sub(count);
            if *value == 0 {
                self.owned.remove(id);
            }
        }
    }

    pub fn owned<'a>(
        &'a self,
        database: &'a ItemsDatabase,
    ) -> impl Iterator<Item = (&str, usize, &Item)> {
        self.owned
            .iter()
            .filter_map(|(id, count)| Some((id.as_str(), *count, database.items.get(id)?)))
    }
}
