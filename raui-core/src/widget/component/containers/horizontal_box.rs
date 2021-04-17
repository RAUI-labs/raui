use crate::{
    widget,
    widget::{
        component::{
            containers::flex_box::{flex_box, FlexBoxProps},
            interactive::navigation::{
                use_nav_container_active, use_nav_item, use_nav_jump_horizontal_step_active,
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
pub struct HorizontalBoxProps {
    #[serde(default)]
    pub separation: Scalar,
    #[serde(default)]
    pub reversed: bool,
    #[serde(default)]
    pub transform: Transform,
}
implement_props_data!(HorizontalBoxProps);

widget_component!(
    #[pre(
        use_nav_container_active,
        use_nav_jump_horizontal_step_active,
        use_nav_item
    )]
    pub fn nav_horizontal_box(key: Key, props: Props, listed_slots: ListedSlots) {
        let props = props
            .clone()
            .without::<NavContainerActive>()
            .without::<NavJumpActive>()
            .without::<NavItemActive>();

        widget! {
            (#{key} horizontal_box: {props} |[listed_slots]|)
        }
    }
);

widget_component!(
    pub fn horizontal_box(key: Key, props: Props, listed_slots: ListedSlots) {
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

        widget! {
            (#{key} flex_box: {props} |[ listed_slots ]|)
        }
    }
);
