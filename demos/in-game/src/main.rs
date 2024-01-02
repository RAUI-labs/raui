mod model;
mod ui;

use model::{
    inventory::{Inventory, ItemsDatabase},
    menu::{Menu, MenuScreen},
    quests::Quests,
    settings::Settings,
};
use raui::prelude::*;
use ui::app::app;

fn main() {
    let app = DeclarativeApp::default()
        .tree(make_widget!(app))
        .view_model(Menu::VIEW_MODEL, Menu::view_model())
        .view_model(Settings::VIEW_MODEL, Settings::view_model())
        .view_model(
            Quests::VIEW_MODEL,
            Quests::view_model("resources/quests.json"),
        )
        .view_model(
            ItemsDatabase::VIEW_MODEL,
            ItemsDatabase::view_model("resources/items.json"),
        )
        .view_model(Inventory::VIEW_MODEL, Inventory::view_model())
        .event(|app, event, _, _| {
            if let Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } = event
            {
                if input.state == ElementState::Pressed {
                    match input.virtual_keycode {
                        Some(VirtualKeyCode::Key1) => {
                            *app.view_models
                                .get_mut(Menu::VIEW_MODEL)
                                .unwrap()
                                .write::<Menu>()
                                .unwrap()
                                .screen = MenuScreen::None;
                            println!("Changed menu: None");
                        }
                        Some(VirtualKeyCode::Key2) => {
                            *app.view_models
                                .get_mut(Menu::VIEW_MODEL)
                                .unwrap()
                                .write::<Menu>()
                                .unwrap()
                                .screen = MenuScreen::Settings;
                            println!("Changed menu: Settings");
                        }
                        Some(VirtualKeyCode::Key3) => {
                            *app.view_models
                                .get_mut(Menu::VIEW_MODEL)
                                .unwrap()
                                .write::<Menu>()
                                .unwrap()
                                .screen = MenuScreen::Quests;
                            println!("Changed menu: Quests");
                        }
                        Some(VirtualKeyCode::Key4) => {
                            *app.view_models
                                .get_mut(Menu::VIEW_MODEL)
                                .unwrap()
                                .write::<Menu>()
                                .unwrap()
                                .screen = MenuScreen::Inventory;
                            println!("Changed menu: Inventory");
                        }
                        Some(VirtualKeyCode::Key5) => {
                            app.view_models
                                .get_mut(Inventory::VIEW_MODEL)
                                .unwrap()
                                .write::<Inventory>()
                                .unwrap()
                                .add("potion", 1);
                            println!("Added item: Potion");
                        }
                        Some(VirtualKeyCode::Key6) => {
                            app.view_models
                                .get_mut(Inventory::VIEW_MODEL)
                                .unwrap()
                                .write::<Inventory>()
                                .unwrap()
                                .add("sword", 1);
                            println!("Added item: Sword");
                        }
                        Some(VirtualKeyCode::Key7) => {
                            app.view_models
                                .get_mut(Inventory::VIEW_MODEL)
                                .unwrap()
                                .write::<Inventory>()
                                .unwrap()
                                .add("shield", 1);
                            println!("Added item: Shield");
                        }
                        Some(VirtualKeyCode::Escape) => {
                            return false;
                        }
                        _ => {}
                    }
                }
            }
            true
        });

    App::new(AppConfig::default().title("In-Game").color([0.2, 0.2, 0.2])).run(app);
}
