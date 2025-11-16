use raui::core::{
    Managed, Scalar,
    view_model::{ViewModel, ViewModelValue},
};

pub struct Settings {
    pub fullscreen: ViewModelValue<bool>,
    pub volume: Managed<ViewModelValue<Scalar>>,
}

impl Settings {
    pub const VIEW_MODEL: &str = "settings";
    const FULLSCREEN: &str = "fullscreen";
    const VOLUME: &str = "volume";

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
