use crate::{
    widget,
    widget::{
        component::interactive::navigation::{
            use_nav_container_active, use_nav_item, use_nav_list_active, NavContainerActive,
            NavItemActive, NavListActive, NavListJumpProps,
        },
        unit::content::{ContentBoxItemNode, ContentBoxNode},
        utils::Transform,
    },
    widget_component, widget_hook,
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

widget_hook! {
    use_nav_switch_box(props) {
        let mut jump = props.read_cloned_or_default::<NavListJumpProps>();
        jump.tabs = true;
        props.write(jump);
    }
}

widget_component! {
    pub nav_switch_box(key, props, listed_slots) [
        use_nav_switch_box,
        use_nav_container_active,
        use_nav_list_active,
        use_nav_item,
    ] {
        let props = props.clone()
            .without::<NavContainerActive>()
            .without::<NavListActive>()
            .without::<NavItemActive>();

        widget!{
            (#{key} switch_box: {props} |[listed_slots]|)
        }
    }
}

widget_component! {
    pub switch_box(id, props, listed_slots) {
        let SwitchBoxProps { active_index, clipping, transform } = props.read_cloned_or_default();
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
                transform,
            }
        }}}
    }
}
