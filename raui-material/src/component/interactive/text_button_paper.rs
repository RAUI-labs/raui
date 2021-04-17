use crate::component::{interactive::button_paper::button_paper, text_paper::text_paper};

widget_component!(
    pub fn text_button_paper(key: Key, props: Props) {
        widget! {
            (#{key} button_paper: {props.clone()} {
                content = (#{"text"} text_paper: {props.clone()})
            })
        }
    }
);
