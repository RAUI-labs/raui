use crate::{
    PropsData, make_widget, pre_hooks,
    widget::{
        component::interactive::navigation::{
            NavContainerActive, NavItemActive, NavJumpActive, use_nav_container_active,
            use_nav_item, use_nav_jump_direction_active,
        },
        context::WidgetContext,
        node::WidgetNode,
        unit::grid::{GridBoxItemLayout, GridBoxItemNode, GridBoxNode},
        utils::Transform,
    },
};
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct GridBoxProps {
    #[serde(default)]
    pub cols: usize,
    #[serde(default)]
    pub rows: usize,
    #[serde(default)]
    pub transform: Transform,
}

#[pre_hooks(use_nav_container_active, use_nav_jump_direction_active, use_nav_item)]
pub fn nav_grid_box(mut context: WidgetContext) -> WidgetNode {
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

    make_widget!(grid_box)
        .key(key)
        .merge_props(props)
        .listed_slots(listed_slots)
        .into()
}

pub fn grid_box(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        props,
        listed_slots,
        ..
    } = context;

    let GridBoxProps {
        cols,
        rows,
        transform,
    } = props.read_cloned_or_default();

    let items = listed_slots
        .into_iter()
        .filter_map(|slot| {
            if let Some(props) = slot.props() {
                let layout = props.read_cloned_or_default::<GridBoxItemLayout>();
                Some(GridBoxItemNode { slot, layout })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    GridBoxNode {
        id: id.to_owned(),
        props: props.clone(),
        items,
        cols,
        rows,
        transform,
    }
    .into()
}
