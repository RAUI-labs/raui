use crate::{
    widget,
    widget::{
        component::interactive::navigation::{
            use_nav_container_active, use_nav_item, use_nav_jump_step_pages_active,
            NavContainerActive, NavItemActive, NavJumpActive,
        },
        unit::content::{ContentBoxItemNode, ContentBoxNode},
        utils::Transform,
    },
    widget_component,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SwitchBoxProps {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_index: Option<usize>,
    #[serde(default)]
    pub clipping: bool,
    #[serde(default)]
    pub transform: Transform,
}
implement_props_data!(SwitchBoxProps);

widget_component!(
    #[pre(use_nav_container_active, use_nav_jump_step_pages_active, use_nav_item)]
    pub fn nav_switch_box(key: Key, props: Props, listed_slots: ListedSlots) {
        let props = props
            .clone()
            .without::<NavContainerActive>()
            .without::<NavJumpActive>()
            .without::<NavItemActive>();

        widget! {
            (#{key} switch_box: {props} |[listed_slots]|)
        }
    }
);

widget_component!(
    pub fn switch_box(id: Id, props: Props, listed_slots: ListedSlots) {
        let SwitchBoxProps {
            active_index,
            clipping,
            transform,
        } = props.read_cloned_or_default();
        let items = if let Some(index) = active_index {
            if let Some(slot) = listed_slots.into_iter().nth(index) {
                vec![ContentBoxItemNode {
                    slot,
                    ..Default::default()
                }]
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
                transform,
            }
        }}}
    }
);
