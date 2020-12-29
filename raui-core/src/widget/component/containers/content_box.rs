use crate::{
    widget,
    widget::unit::content::{ContentBoxItemLayout, ContentBoxItemNode, ContentBoxNode},
    widget_component,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ContentBoxProps {
    #[serde(default)]
    pub clipping: bool,
}
implement_props_data!(ContentBoxProps, "ContentBoxProps");

widget_component! {
    pub content_box(id, props, listed_slots) {
        let ContentBoxProps { clipping } = props.read_cloned_or_default();
        let items = listed_slots.into_iter().map(|slot| {
            let layout = slot
                .props()
                .expect("WidgetNode has no Props")
                .read_cloned_or_default::<ContentBoxItemLayout>();
            ContentBoxItemNode {
                slot,
                layout,
            }
        }).collect::<Vec<_>>();

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
