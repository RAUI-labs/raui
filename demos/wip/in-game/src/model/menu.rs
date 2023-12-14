use raui::prelude::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum MenuScreen {
    #[default]
    None,
    Settings,
    Inventory,
    Quests,
}

pub struct Menu {
    pub screen: ViewModelValue<MenuScreen>,
}

impl Menu {
    pub const VIEW_MODEL: &'static str = "menu";
    const SCREEN: &'static str = "screen";

    pub fn view_model() -> ViewModel {
        ViewModel::produce(|properties| Self {
            screen: ViewModelValue::new(Default::default(), properties.notifier(Self::SCREEN)),
        })
    }
}
