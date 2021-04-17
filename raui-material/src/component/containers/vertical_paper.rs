use crate::component::containers::paper::paper;
use raui_core::prelude::*;

widget_component!(
    pub fn nav_vertical_paper(key: Key, props: Props, listed_slots: ListedSlots) {
        widget! {
            (#{key} paper: {props.clone()} [
                (#{"vertical"} nav_vertical_box: {props.clone()} |[ listed_slots ]|)
            ])
        }
    }
);

widget_component!(
    pub fn vertical_paper(key: Key, props: Props, listed_slots: ListedSlots) {
        widget! {
            (#{key} paper: {props.clone()} [
                (#{"vertical"} vertical_box: {props.clone()} |[ listed_slots ]|)
            ])
        }
    }
);
