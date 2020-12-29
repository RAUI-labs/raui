use crate::component::containers::paper::paper;
use raui_core::prelude::*;

widget_component! {
    pub vertical_paper(key, props, listed_slots) {
        widget! {
            (#{key} paper: {props.clone()} [
                (#{"vertical"} vertical_box: {props.clone()} |[ listed_slots ]|)
            ])
        }
    }
}
