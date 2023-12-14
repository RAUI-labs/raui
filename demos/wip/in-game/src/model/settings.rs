use raui::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct SettingsData {
    fullscreen: bool,
}

pub struct Settings {
    pub fullscreen: ViewModelValue<bool>,
}

impl Settings {
    pub const VIEW_MODEL: &'static str = "settings";
    const FULLSCREEN: &'static str = "fullscreen";

    pub fn view_model() -> ViewModel {
        ViewModel::produce(|properties| Self {
            fullscreen: ViewModelValue::new(false, properties.notifier(Self::FULLSCREEN)),
        })
    }
}
