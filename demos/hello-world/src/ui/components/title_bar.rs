use raui_core::prelude::*;

widget_component! {
    pub title_bar(id, key, state) [use_button_notified_state, use_text_input_notified_state] {
        let ButtonProps { selected, trigger, .. } = state.read_cloned_or_default();
        let TextInputProps { text, cursor_position, focused, .. } = state.read_cloned_or_default();
        let text = text.trim();
        let text = if text.is_empty() {
            "> Focus here and start typing...".to_owned()
        } else if focused {
            if cursor_position < text.len() {
                format!("{}|{}", &text[..cursor_position], &text[cursor_position..])
            } else {
                format!("{}|", text)
            }
        } else {
            text.to_owned()
        };
        let text_props = TextBoxProps {
            text,
            width: TextBoxSizeValue::Fill,
            height: TextBoxSizeValue::Exact(32.0),
            alignment: TextBoxAlignment::Center,
            font: TextBoxFont {
                name: "verdana".to_owned(),
                size: 32.0,
            },
            color: if trigger {
                Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 }
            } else if selected {
                Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 }
            } else if focused {
                Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 }
            } else {
                Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }
            },
            ..Default::default()
        };
        let input_props = Props::new(NavItemActive)
            .with(TextInputNotifyProps(id.to_owned().into()))
            .with(ButtonNotifyProps(id.to_owned().into()));

        widget! {
            (#{key} input_field: {input_props} {
                content = (#{"text"} text_box: {text_props})
            })
        }
    }
}
