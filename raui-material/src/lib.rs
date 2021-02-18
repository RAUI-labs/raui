#[macro_use]
extern crate raui_core;

pub mod component;
pub mod theme;

use raui_core::application::Application;

pub fn setup(app: &mut Application) {
    app.register_props::<theme::ThemeProps>("ThemeProps");
    app.register_props::<theme::ThemedWidgetProps>("ThemedWidgetProps");
    app.register_props::<theme::ThemedWidgetProps>("ThemedWidgetProps");
    app.register_props::<component::containers::paper::PaperProps>("PaperProps");
    app.register_props::<component::interactive::text_field_paper::TextFieldPaperProps>(
        "TextFieldPaperProps",
    );
    app.register_props::<component::icon_paper::IconPaperProps>("IconPaperProps");
    app.register_props::<component::switch_paper::SwitchPaperProps>("SwitchPaperProps");
    app.register_props::<component::text_paper::TextPaperProps>("TextPaperProps");

    app.register_component("paper", component::containers::paper::paper);
    app.register_component("flex_paper", component::containers::flex_paper::flex_paper);
    app.register_component(
        "vertical_paper",
        component::containers::vertical_paper::vertical_paper,
    );
    app.register_component(
        "horizontal_paper",
        component::containers::horizontal_paper::horizontal_paper,
    );
    app.register_component("grid_paper", component::containers::grid_paper::grid_paper);
    app.register_component("wrap_paper", component::containers::wrap_paper::wrap_paper);
    app.register_component(
        "button_paper",
        component::interactive::button_paper::button_paper,
    );
    app.register_component(
        "icon_button_paper",
        component::interactive::icon_button_paper::icon_button_paper,
    );
    app.register_component(
        "switch_button_paper",
        component::interactive::switch_button_paper::switch_button_paper,
    );
    app.register_component(
        "text_button_paper",
        component::interactive::text_button_paper::text_button_paper,
    );
    app.register_component(
        "text_field_paper",
        component::interactive::text_field_paper::text_field_paper,
    );
    app.register_component("icon_paper", component::icon_paper::icon_paper);
    app.register_component("switch_paper", component::switch_paper::switch_paper);
    app.register_component("text_paper", component::text_paper::text_paper);
}

pub mod prelude {
    pub use crate::{
        component::{
            containers::{
                flex_paper::*, grid_paper::*, horizontal_paper::*, paper::*, vertical_paper::*,
                wrap_paper::*,
            },
            icon_paper::*,
            interactive::{
                button_paper::*, icon_button_paper::*, switch_button_paper::*,
                text_button_paper::*, text_field_paper::*,
            },
            switch_paper::*,
            text_paper::*,
        },
        theme::*,
    };
}
