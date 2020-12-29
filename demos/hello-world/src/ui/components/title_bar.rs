use raui_core::prelude::*;

widget_component! {
    title_bar_input(key, props) {
        let ButtonProps { selected, trigger, .. } = props.read_cloned_or_default();
        let InputFieldProps { text, cursor_position, .. } = props.read_cloned_or_default();
        let text = text.trim();
        let text = if text.is_empty() {
            "> Hover here and start typing...".to_owned()
        } else if selected {
            if cursor_position < text.len() {
                format!("{}|{}", &text[..cursor_position], &text[cursor_position..])
            } else {
                format!("{}|", text)
            }
        } else {
            text.to_owned()
        };

        widget! {
            (#{key} text_box: {TextBoxProps {
                text,
                width: TextBoxSizeValue::Fill,
                height: TextBoxSizeValue::Exact(32.0),
                alignment: TextBoxAlignment::Center,
                font: TextBoxFont {
                    name: "verdana".to_owned(),
                    size: 48.0,
                },
                color: if trigger {
                    Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 }
                } else if selected {
                    Color { r: 0.0, g: 0.0, b: 0.0, a: 0.85 }
                } else {
                    Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }
                },
                ..Default::default()
            }})
        }
    }
}

widget_component! {
    pub title_bar(key) {
        widget! {
            (#{key} input_field {
                content = (title_bar_input)
            })
        }
    }
}
