use crate::{
    unpack_named_slots, widget,
    widget::component::interactive::button::{button, ButtonProps, TextChange},
    widget_component, widget_hook,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InputFieldProps {
    pub text: String,
    pub cursor_position: usize,
    pub allow_new_line: bool,
}
implement_props_data!(InputFieldProps, "InputFieldProps");

widget_hook! {
    use_input_field life_cycle {
        life_cycle.mount(|id, props, state, _, _| {
            let props = props.read_cloned_or_default::<InputFieldProps>();
            drop(state.write(props));
        });

        life_cycle.change(|_, props, state, _, signals| {
            let props = props.read_cloned_or_default::<ButtonProps>();
            let mut data = match state.read::<InputFieldProps>() {
                Ok(state) => state.clone(),
                Err(_) => InputFieldProps::default(),
            };
            if !props.text.is_empty() {
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
                drop(state.write(data));
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
        }

        content
    }
}

widget_component! {
    pub input_field(key, props, named_slots) {
        unpack_named_slots!(named_slots => content);

        widget! {
            (#{key} button: {props.clone()} {
                content = (input_field_content {
                    content = {content}
                })
            })
        }
    }
}
