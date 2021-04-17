use crate::{
    widget,
    widget::{
        component::interactive::navigation::{
            use_nav_container_active, use_nav_item, use_nav_jump_direction_active,
            NavContainerActive, NavItemActive, NavJumpActive,
        },
        unit::grid::{GridBoxItemLayout, GridBoxItemNode, GridBoxNode},
        utils::Transform,
    },
    widget_component,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GridBoxProps {
    #[serde(default)]
    pub cols: usize,
    #[serde(default)]
    pub rows: usize,
    #[serde(default)]
    pub transform: Transform,
}
implement_props_data!(GridBoxProps);

widget_component!(
    #[pre(use_nav_container_active, use_nav_jump_direction_active, use_nav_item)]
    pub fn nav_grid_box(key: Key, props: Props, listed_slots: ListedSlots) {
        let props = props
            .clone()
            .without::<NavContainerActive>()
            .without::<NavJumpActive>()
            .without::<NavItemActive>();

        widget! {
            (#{key} grid_box: {props} |[listed_slots]|)
        }
    }
);

widget_component!(
    pub fn grid_box(id: Id, props: Props, listed_slots: ListedSlots) {
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

        widget! {{{
            GridBoxNode {
                id: id.to_owned(),
                props: props.clone(),
                items,
                cols,
                rows,
                transform,
            }
        }}}
    }
);
