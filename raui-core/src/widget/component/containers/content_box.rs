use crate::{
    widget,
    widget::{
        unit::content::{ContentBoxItemLayout, ContentBoxItemNode, ContentBoxNode},
        utils::Transform,
    },
    widget_component,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ContentBoxProps {
    #[serde(default)]
    pub clipping: bool,
    #[serde(default)]
    pub transform: Transform,
}
implement_props_data!(ContentBoxProps);

widget_component! {
    pub content_box(id, props, listed_slots) {
        let ContentBoxProps { clipping, transform } = props.read_cloned_or_default();
        let items = listed_slots.into_iter().filter_map(|slot| {
            if let Some(props) = slot.props() {
                let layout = props.read_cloned_or_default::<ContentBoxItemLayout>();
                Some(ContentBoxItemNode {
                    slot,
                    layout,
                })
            } else {
                None
            }
        }).collect::<Vec<_>>();

        widget! {{{
            ContentBoxNode {
                id: id.to_owned(),
                props: props.clone(),
                items,
                clipping,
                transform,
            }
        }}}
    }
}
