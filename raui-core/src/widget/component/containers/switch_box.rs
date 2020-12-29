use crate::{
    widget,
    widget::unit::content::{ContentBoxItemNode, ContentBoxNode},
    widget_component,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SwitchBoxProps {
    #[serde(default)]
    pub active_index: Option<usize>,
    #[serde(default)]
    pub clipping: bool,
}
implement_props_data!(SwitchBoxProps, "SwitchBoxProps");

widget_component! {
    pub switch_box(id, props, listed_slots) {
        let SwitchBoxProps { active_index, clipping } = props.read_cloned_or_default();
        let items = if let Some(index) = active_index {
            if let Some(slot) = listed_slots.into_iter().nth(index) {
                vec![
                    ContentBoxItemNode {
                        slot,
                        ..Default::default()
                    }
                ]
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        widget! {{{
            ContentBoxNode {
                id: id.to_owned(),
                props: props.clone(),
                items,
                clipping,
            }
        }}}
    }
}
