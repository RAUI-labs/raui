use crate::component::containers::paper::paper;
use raui_core::prelude::*;

widget_component!(
    pub fn wrap_paper(key: Key, props: Props, (content,): NamedSlots) {
        widget! {
            (#{key} paper: {props.clone()} [
                (#{"wrap"} wrap_box: {props.clone()} {
                    content = {content}
                })
            ])
        }
    }
);
