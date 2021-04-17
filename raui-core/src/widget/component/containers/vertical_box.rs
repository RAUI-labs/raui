use crate::{
    widget,
    widget::{
        component::{
            containers::flex_box::{flex_box, FlexBoxProps},
            interactive::navigation::{
                use_nav_container_active, use_nav_item, use_nav_jump_vertical_step_active,
                NavContainerActive, NavItemActive, NavJumpActive,
            },
        },
        unit::flex::FlexBoxDirection,
        utils::Transform,
    },
    widget_component, Scalar,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct VerticalBoxProps {
    #[serde(default)]
    pub separation: Scalar,
    #[serde(default)]
    pub reversed: bool,
    #[serde(default)]
    pub transform: Transform,
}
implement_props_data!(VerticalBoxProps);

widget_component!(
    #[pre(
        use_nav_container_active,
        use_nav_jump_vertical_step_active,
        use_nav_item
    )]
    pub fn nav_vertical_box(key: Key, props: Props, listed_slots: ListedSlots) {
        let props = props
            .clone()
            .without::<NavContainerActive>()
            .without::<NavJumpActive>()
            .without::<NavItemActive>();

        widget! {
            (#{key} vertical_box: {props} |[listed_slots]|)
        }
    }
);

widget_component!(
    pub fn vertical_box(key: Key, props: Props, listed_slots: ListedSlots) {
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
);
