use raui::prelude::*;

pub struct AppData {
    pub input: Managed<ViewModelValue<String>>,
}

impl AppData {
    pub const VIEW_MODEL: &'static str = "app-data";
    pub const INPUT: &'static str = "input";

    pub fn new(properties: &mut ViewModelProperties) -> Self {
        Self {
            input: Managed::new(ViewModelValue::new(
                Default::default(),
                properties.notifier(Self::INPUT),
            )),
        }
    }
}
