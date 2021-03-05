use crate::{
    widget,
    widget::{
        component::{
            containers::flex_box::{flex_box, FlexBoxProps},
            interactive::navigation::{
                use_nav_container_active, use_nav_item, use_nav_list_active, NavContainerActive,
                NavItemActive, NavListActive, NavListDirection, NavListJumpProps,
            },
        },
        unit::flex::FlexBoxDirection,
        utils::Transform,
    },
    widget_component, widget_hook, Scalar,
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

widget_hook! {
    use_nav_horizontal_box(props) {
        let reversed = props.map_or_default::<HorizontalBoxProps, _, _>(|p| p.reversed);
        let mut jump = props.read_cloned_or_default::<NavListJumpProps>();
        jump.direction = if reversed {
            NavListDirection::HorizontalRightToLeft
        } else {
            NavListDirection::HorizontalLeftToRight
        };
        props.write(jump);
    }
}

widget_component! {
    pub nav_horizontal_box(key, props, listed_slots) [
        use_nav_horizontal_box,
        use_nav_container_active,
        use_nav_list_active,
        use_nav_item,
    ] {
        let props = props.clone()
            .without::<NavContainerActive>()
            .without::<NavListActive>()
            .without::<NavItemActive>();

        widget!{
            (#{key} horizontal_box: {props} |[listed_slots]|)
        }
    }
}

widget_component! {
    pub horizontal_box(key, props, listed_slots) {
        let HorizontalBoxProps { separation, reversed, transform } = props.read_cloned_or_default();
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
}
