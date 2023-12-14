use super::{inventory::inventory, quests::quests, settings::settings};
use crate::model::menu::{Menu, MenuScreen};
use raui::prelude::*;

pub fn app(context: WidgetContext) -> WidgetNode {
    let menu = context
        .view_models
        .view_model(Menu::VIEW_MODEL)
        .unwrap()
        .read::<Menu>()
        .unwrap();

    match *menu.screen {
        MenuScreen::None => Default::default(),
        MenuScreen::Settings => make_widget!(settings).into(),
        MenuScreen::Inventory => make_widget!(inventory).into(),
        MenuScreen::Quests => make_widget!(quests).into(),
    }
}
