use crate::{
    make_widget, pre_hooks,
    widget::{
        component::interactive::navigation::{
            use_nav_container_active, use_nav_item, use_nav_jump_step_pages_active,
            NavContainerActive, NavItemActive, NavJumpActive,
        },
        context::WidgetContext,
        node::WidgetNode,
        unit::content::{ContentBoxItemNode, ContentBoxNode},
        utils::Transform,
    },
    PropsData,
};
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct SwitchBoxProps {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_index: Option<usize>,
    #[serde(default)]
    pub clipping: bool,
    #[serde(default)]
    pub transform: Transform,
}

#[pre_hooks(use_nav_container_active, use_nav_jump_step_pages_active, use_nav_item)]
pub fn nav_switch_box(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        props,
        listed_slots,
        ..
    } = context;

    let props = props
        .clone()
        .without::<NavContainerActive>()
        .without::<NavJumpActive>()
        .without::<NavItemActive>();

    make_widget!(switch_box)
        .key(key)
        .merge_props(props)
        .listed_slots(listed_slots)
        .into()
}

pub fn switch_box(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        props,
        listed_slots,
        ..
    } = context;

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

    ContentBoxNode {
        id: id.to_owned(),
        props: props.clone(),
        items,
        clipping,
        transform,
    }
    .into()
}
