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
    owned: ViewModelValue<Vec<String>>,
}

impl Inventory {
    pub const VIEW_MODEL: &'static str = "items";
    const OWNED: &'static str = "owned";

    pub fn view_model() -> ViewModel {
        ViewModel::produce(|properties| Self {
            owned: ViewModelValue::new(Default::default(), properties.notifier(Self::OWNED)),
        })
    }

    pub fn owned<'a>(&'a self, database: &'a ItemsDatabase) -> impl Iterator<Item = (&str, &Item)> {
        self.owned
            .iter()
            .filter_map(|id| Some((id.as_str(), database.items.get(id)?)))
    }
}
