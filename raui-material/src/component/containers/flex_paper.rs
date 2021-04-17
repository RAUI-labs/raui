use crate::component::containers::paper::paper;
use raui_core::prelude::*;

widget_component!(
    pub fn nav_flex_paper(key: Key, props: Props, listed_slots: ListedSlots) {
        widget! {
            (#{key} paper: {props.clone()} [
                (#{"flex"} nav_flex_box: {props.clone()} |[ listed_slots ]|)
            ])
        }
    }
);

widget_component!(
    pub fn flex_paper(key: Key, props: Props, listed_slots: ListedSlots) {
        widget! {
            (#{key} paper: {props.clone()} [
                (#{"flex"} flex_box: {props.clone()} |[ listed_slots ]|)
            ])
        }
    }
);
