use crate::{
    widget,
    widget::{
        unit::content::{ContentBoxItemLayout, ContentBoxItemNode, ContentBoxNode},
        utils::Rect,
    },
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
        let items = listed_slots.into_iter().map(|slot| {
            let layout = match slot
                .props()
                .expect("WidgetNode has no Props")
                .read::<ContentBoxItemLayout>() {
                    Ok(layout) => layout.clone(),
                    Err(_) => ContentBoxItemLayout {
                        anchors: Rect {
                            left: 0.0,
                            right: 1.0,
                            top: 0.0,
                            bottom: 1.0,
                        },
                        ..Default::default()
                    },
                };
            ContentBoxItemNode {
                slot,
                layout,
            }
        }).collect::<Vec<_>>();
        let ContentBoxProps { clipping } = props.read_cloned_or_default();

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
