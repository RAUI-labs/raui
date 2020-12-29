use crate::component::{interactive::button_paper::button_paper, switch_paper::switch_paper};

widget_component! {
    pub switch_button_paper(key, props) {
        widget! {
            (#{key} button_paper: {props.clone()} {
                content = (#{"switch"} switch_paper: {props.clone()})
            })
        }
    }
}
