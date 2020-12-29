use crate::component::containers::paper::paper;
use raui_core::prelude::*;

widget_component! {
    pub grid_paper(key, props, listed_slots) {
        widget! {
            (#{key} paper: {props.clone()} [
                (#{"grid"} grid_box: {props.clone()} |[ listed_slots ]|)
            ])
        }
    }
}
