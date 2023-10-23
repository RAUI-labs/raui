use crate::{
    make_widget, pre_hooks,
    widget::{
        component::{
            containers::flex_box::{flex_box, FlexBoxProps},
            interactive::navigation::{
                use_nav_container_active, use_nav_item, use_nav_jump_horizontal_step_active,
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
pub struct HorizontalBoxProps {
    #[serde(default)]
    pub separation: Scalar,
    #[serde(default)]
    pub reversed: bool,
    #[serde(default)]
    pub transform: Transform,
}

#[pre_hooks(
    use_nav_container_active,
    use_nav_jump_horizontal_step_active,
    use_nav_item
)]
pub fn nav_horizontal_box(mut context: WidgetContext) -> WidgetNode {
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

    make_widget!(horizontal_box)
        .key(key)
        .merge_props(props)
        .listed_slots(listed_slots)
        .into()
}

pub fn horizontal_box(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        props,
        listed_slots,
        ..
    } = context;

    let HorizontalBoxProps {
        separation,
        reversed,
        transform,
    } = props.read_cloned_or_default();

    let props = props.clone().with(FlexBoxProps {
        direction: if reversed {
            FlexBoxDirection::HorizontalRightToLeft
        } else {
            FlexBoxDirection::HorizontalLeftToRight
        },
        separation,
        wrap: false,
        transform,
    });

    make_widget!(flex_box)
        .key(key)
        .merge_props(props)
        .listed_slots(listed_slots)
        .into()
}
