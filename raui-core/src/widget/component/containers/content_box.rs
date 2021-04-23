use crate::{
    pre_hooks, widget,
    widget::{
        component::interactive::navigation::{
            use_nav_container_active, use_nav_item, use_nav_jump_direction_active,
            NavContainerActive, NavItemActive, NavJumpActive,
        },
        context::WidgetContext,
        node::WidgetNode,
        unit::content::{ContentBoxItemLayout, ContentBoxItemNode, ContentBoxNode},
        utils::Transform,
    },
    PropsData,
};
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct ContentBoxProps {
    #[serde(default)]
    pub clipping: bool,
    #[serde(default)]
    pub transform: Transform,
}

#[pre_hooks(use_nav_container_active, use_nav_jump_direction_active, use_nav_item)]
pub fn nav_content_box(mut context: WidgetContext) -> WidgetNode {
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

    widget! {
        (#{key} content_box: {props} |[listed_slots]|)
    }
}

pub fn content_box(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        props,
        listed_slots,
        ..
    } = context;

    let ContentBoxProps {
        clipping,
        transform,
    } = props.read_cloned_or_default();

    let items = listed_slots
        .into_iter()
        .filter_map(|slot| {
            if let Some(props) = slot.props() {
                let layout = props.read_cloned_or_default::<ContentBoxItemLayout>();
                Some(ContentBoxItemNode { slot, layout })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

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
