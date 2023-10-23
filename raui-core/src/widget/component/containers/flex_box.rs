use crate::{
    make_widget, pre_hooks,
    widget::{
        component::interactive::navigation::{
            use_nav_container_active, use_nav_item, use_nav_jump, NavContainerActive,
            NavItemActive, NavJumpActive,
        },
        context::WidgetContext,
        node::WidgetNode,
        unit::flex::{FlexBoxDirection, FlexBoxItemLayout, FlexBoxItemNode, FlexBoxNode},
        utils::Transform,
    },
    PropsData, Scalar,
};
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct FlexBoxProps {
    #[serde(default)]
    pub direction: FlexBoxDirection,
    #[serde(default)]
    pub separation: Scalar,
    #[serde(default)]
    pub wrap: bool,
    #[serde(default)]
    pub transform: Transform,
}

#[pre_hooks(use_nav_container_active, use_nav_jump, use_nav_item)]
pub fn nav_flex_box(mut context: WidgetContext) -> WidgetNode {
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

    make_widget!(flex_box)
        .key(key)
        .merge_props(props)
        .listed_slots(listed_slots)
        .into()
}

pub fn flex_box(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        props,
        listed_slots,
        ..
    } = context;

    let FlexBoxProps {
        direction,
        separation,
        wrap,
        transform,
    } = props.read_cloned_or_default();

    let items = listed_slots
        .into_iter()
        .filter_map(|slot| {
            if let Some(props) = slot.props() {
                let layout = props.read_cloned_or_default::<FlexBoxItemLayout>();
                Some(FlexBoxItemNode { slot, layout })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    FlexBoxNode {
        id: id.to_owned(),
        props: props.clone(),
        items,
        direction,
        separation,
        wrap,
        transform,
    }
    .into()
}
