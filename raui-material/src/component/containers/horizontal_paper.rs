use crate::component::containers::paper::paper;
use raui_core::prelude::*;

widget_component! {
    pub fn nav_horizontal_paper(key: Key, props: Props, listed_slots: ListedSlots) {
        widget! {
            (#{key} paper: {props.clone()} [
                (#{"horizontal"} nav_horizontal_box: {props.clone()} |[ listed_slots ]|)
            ])
        }
    }
}

widget_component! {
    pub fn horizontal_paper(key: Key, props: Props, listed_slots: ListedSlots) {
        widget! {
            (#{key} paper: {props.clone()} [
                (#{"horizontal"} horizontal_box: {props.clone()} |[ listed_slots ]|)
            ])
        }
    }
}
