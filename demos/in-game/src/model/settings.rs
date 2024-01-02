use raui::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct SettingsData {
    fullscreen: bool,
}

pub struct Settings {
    pub fullscreen: ViewModelValue<bool>,
    pub volume: Managed<ViewModelValue<Scalar>>,
}

impl Settings {
    pub const VIEW_MODEL: &'static str = "settings";
    const FULLSCREEN: &'static str = "fullscreen";
    const VOLUME: &'static str = "volume";

    pub fn view_model() -> ViewModel {
        ViewModel::produce(|properties| Self {
            fullscreen: ViewModelValue::new(false, properties.notifier(Self::FULLSCREEN)),
            volume: Managed::new(ViewModelValue::new(
                100.0,
                properties.notifier(Self::VOLUME),
            )),
        })
    }
}
