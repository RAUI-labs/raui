use raui::prelude::*;

#[pre_hooks(use_button_notified_state, use_text_input_notified_state)]
pub fn title_bar(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext { id, key, state, .. } = context;

    let ButtonProps {
        selected, trigger, ..
    } = state.read_cloned_or_default();
    let TextInputProps {
        text,
        cursor_position,
        focused,
        ..
    } = state.read_cloned_or_default();
    let text = if text.trim().is_empty() {
        "> Focus here and start typing...".to_owned()
    } else if focused {
        if cursor_position < text.len() {
            format!("{}|{}", &text[..cursor_position], &text[cursor_position..])
        } else {
            format!("{}|", text)
        }
    } else {
        text
    };
    let text_props = TextBoxProps {
        text,
        width: TextBoxSizeValue::Fill,
        height: TextBoxSizeValue::Exact(32.0),
        font: TextBoxFont {
            name: "./resources/verdana.ttf".to_owned(),
            size: 32.0,
        },
        color: if trigger {
            Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }
        } else if selected {
            Color {
                r: 0.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            }
        } else if focused {
            Color {
                r: 0.0,
                g: 0.0,
                b: 1.0,
                a: 1.0,
            }
        } else {
            Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }
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
