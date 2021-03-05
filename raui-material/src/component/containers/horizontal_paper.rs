use crate::component::containers::paper::paper;
use raui_core::prelude::*;

widget_component! {
    pub nav_horizontal_paper(key, props, listed_slots) {
        widget! {
            (#{key} paper: {props.clone()} [
                (#{"horizontal"} nav_horizontal_box: {props.clone()} |[ listed_slots ]|)
            ])
        }
    }
}

widget_component! {
    pub horizontal_paper(key, props, listed_slots) {
        widget! {
            (#{key} paper: {props.clone()} [
                (#{"horizontal"} horizontal_box: {props.clone()} |[ listed_slots ]|)
            ])
        }
    }
}
