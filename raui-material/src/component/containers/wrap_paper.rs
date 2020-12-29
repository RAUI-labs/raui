use crate::component::containers::paper::paper;
use raui_core::prelude::*;

widget_component! {
    pub wrap_paper(key, props, named_slots) {
        unpack_named_slots!(named_slots => content);

        widget! {
            (#{key} paper: {props.clone()} [
                (#{"wrap"} wrap_box: {props.clone()} {
                    content = {content}
                })
            ])
        }
    }
}
