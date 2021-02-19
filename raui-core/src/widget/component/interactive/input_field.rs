use crate::{
    unpack_named_slots, widget,
    widget::{
        component::interactive::button::{button, ButtonProps, ButtonSettingsProps, TextChange},
        WidgetId,
    },
    widget_component, widget_hook,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InputFieldProps {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub cursor_position: usize,
    #[serde(default)]
    pub allow_new_line: bool,
}
implement_props_data!(InputFieldProps);

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InputFieldMessage {
    #[serde(default)]
    pub sender: WidgetId,
    #[serde(default)]
    pub data: InputFieldProps,
}

widget_hook! {
    use_input_field(life_cycle) {
        life_cycle.mount(|context| {
            let props = context.props.read_cloned_or_default::<InputFieldProps>();
            drop(context.state.write(props));
        });

        life_cycle.change(|context| {
            let ButtonSettingsProps { disabled, notify } = context.props.read_cloned_or_default();
            let props = context.props.read_cloned_or_default::<ButtonProps>();
            let mut data = match context.state.read::<InputFieldProps>() {
                Ok(state) => state.clone(),
                Err(_) => InputFieldProps::default(),
            };
            if !disabled && !props.text.is_empty() {
                for change in &props.text {
                    match change {
                        TextChange::InsertCharacter(c) => {
                            if !c.is_control() {
                                data.cursor_position = data.cursor_position.min(data.text.len());
                                data.text.insert(data.cursor_position, *c);
                                data.cursor_position += 1;
                            }
                        }
                        TextChange::MoveCursorLeft => if data.cursor_position > 0 {
                            data.cursor_position -= 1;
                        }
                        TextChange::MoveCursorRight => if data.cursor_position < data.text.len() {
                            data.cursor_position += 1;
                        }
                        TextChange::MoveCursorStart => data.cursor_position = 0,
                        TextChange::MoveCursorEnd => data.cursor_position = data.text.len(),
                        TextChange::DeleteLeft => {
                            if data.cursor_position > 0 && data.cursor_position <= data.text.len() {
                                data.cursor_position -= 1;
                                data.text.remove(data.cursor_position);
                            }
                        }
                        TextChange::DeleteRight => if data.cursor_position < data.text.len() {
                            data.text.remove(data.cursor_position);
                        }
                        TextChange::NewLine => if data.allow_new_line {
                            data.cursor_position = data.cursor_position.min(data.text.len());
                            data.text.insert(data.cursor_position, '\n');
                            data.cursor_position += 1;
                        }
                    }
                    data.cursor_position = data.cursor_position.min(data.text.len());
                }
                if let Some(notify) = notify {
                    context.messenger.write(notify, InputFieldMessage {
                        sender: context.id.to_owned(),
                        data: data.clone(),
                    });
                }
                drop(context.state.write(data));
            }
        });
    }
}

widget_component! {
    pub input_field_content(props, state, named_slots) [use_input_field] {
        unpack_named_slots!(named_slots => content);
        if let Some(content_props) = content.props_mut() {
            if let Ok(s) = state.read::<InputFieldProps>() {
                content_props.write(s.clone());
            };
            if let Ok(p) = props.read::<ButtonProps>() {
                content_props.write(p.clone());
            };
            if let Ok(p) = props.read::<ButtonSettingsProps>() {
                content_props.write(p.clone());
            };
        }

        content
    }
}

widget_component! {
    pub input_field(key, props, named_slots) {
        unpack_named_slots!(named_slots => content);

        widget! {
            (#{key} button: {props.clone()} {
                content = (input_field_content: {props.clone()} {
                    content = {content}
                })
            })
        }
    }
}
