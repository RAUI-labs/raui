use crate::{
    PropsData, Scalar, make_widget, pre_hooks,
    widget::{
        component::{
            containers::flex_box::{FlexBoxProps, flex_box},
            interactive::navigation::{
                NavContainerActive, NavItemActive, NavJumpActive, use_nav_container_active,
                use_nav_item, use_nav_jump_vertical_step_active,
            },
        },
        context::WidgetContext,
        node::WidgetNode,
        unit::flex::{FlexBoxDirection, FlexBoxItemLayout},
        utils::Transform,
    },
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
    pub override_slots_layout: Option<FlexBoxItemLayout>,
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

    make_widget!(vertical_box)
        .key(key)
        .merge_props(props)
        .listed_slots(listed_slots)
        .into()
}

pub fn vertical_box(context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        props,
        mut listed_slots,
        ..
    } = context;

    let VerticalBoxProps {
        separation,
        reversed,
        override_slots_layout,
        transform,
    } = props.read_cloned_or_default();

    if let Some(layout) = override_slots_layout {
        for slot in &mut listed_slots {
            if let Some(props) = slot.props_mut() {
                props.write(layout.to_owned());
            }
        }
    }

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

    make_widget!(flex_box)
        .key(key)
        .merge_props(props)
        .listed_slots(listed_slots)
        .into()
}
