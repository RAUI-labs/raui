use crate::{
    pre_hooks, widget,
    widget::{
        component::{
            containers::flex_box::{flex_box, FlexBoxProps},
            interactive::navigation::{
                use_nav_container_active, use_nav_item, use_nav_jump_vertical_step_active,
                NavContainerActive, NavItemActive, NavJumpActive,
            },
        },
        context::WidgetContext,
        node::WidgetNode,
        unit::flex::FlexBoxDirection,
        utils::Transform,
    },
    PropsData, Scalar,
};
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct VerticalBoxProps {
    #[serde(default)]
    pub separation: Scalar,
    #[serde(default)]
    pub reversed: bool,
    #[serde(default)]
    pub transform: Transform,
}

#[pre_hooks(
    use_nav_container_active,
    use_nav_jump_vertical_step_active,
    use_nav_item
)]
pub fn nav_vertical_box(mut context: WidgetContext) -> WidgetNode {
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
        (#{key} vertical_box: {props} |[listed_slots]|)
    }
}

pub fn vertical_box(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        props,
        listed_slots,
        ..
    } = context;

    let VerticalBoxProps {
        separation,
        reversed,
        transform,
    } = props.read_cloned_or_default();

    let props = props.clone().with(FlexBoxProps {
        direction: if reversed {
            FlexBoxDirection::VerticalBottomToTop
        } else {
            FlexBoxDirection::VerticalTopToBottom
        },
        separation,
        wrap: false,
        transform,
    });

    widget! {
        (#{key} flex_box: {props} |[ listed_slots ]|)
    }
}
