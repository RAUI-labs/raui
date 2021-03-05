use crate::{
    widget,
    widget::{
        component::interactive::navigation::{
            use_nav_container_active, use_nav_item, NavContainerActive, NavItemActive,
        },
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
    pub nav_content_box(key, props, listed_slots) [use_nav_container_active, use_nav_item] {
        let props = props.clone()
            .without::<NavContainerActive>()
            .without::<NavItemActive>();

        widget!{
            (#{key} content_box: {props} |[listed_slots]|)
        }
    }
}

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
