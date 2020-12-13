use raui_core::prelude::*;

widget_component! {
    pub title_bar(id) {
        widget! {{{
            TextBox {
                id: id.to_owned(),
                text: "Hello, World!".to_owned(),
                alignment: TextBoxAlignment::Center,
                font: TextBoxFont {
                    name: "verdana".to_owned(),
                    size: 48.0,
                    ..Default::default()
                },
                color: Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
                ..Default::default()
            }
        }}}
    }
}
