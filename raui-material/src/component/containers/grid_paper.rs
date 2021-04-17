use crate::component::containers::paper::paper;
use raui_core::prelude::*;

widget_component!(
    pub fn nav_grid_paper(key: Key, props: Props, listed_slots: ListedSlots) {
        widget! {
            (#{key} paper: {props.clone()} [
                (#{"grid"} nav_grid_box: {props.clone()} |[ listed_slots ]|)
            ])
        }
    }
);

widget_component!(
    pub fn grid_paper(key: Key, props: Props, listed_slots: ListedSlots) {
        widget! {
            (#{key} paper: {props.clone()} [
                (#{"grid"} grid_box: {props.clone()} |[ listed_slots ]|)
            ])
        }
    }
);
