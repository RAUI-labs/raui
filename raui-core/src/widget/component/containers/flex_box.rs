use crate::{
    widget,
    widget::{
        component::interactive::navigation::{
            use_nav_container_active, use_nav_item, use_nav_list_active, NavContainerActive,
            NavItemActive, NavListActive, NavListDirection, NavListJumpProps,
        },
        unit::flex::{FlexBoxDirection, FlexBoxItemLayout, FlexBoxItemNode, FlexBoxNode},
        utils::Transform,
    },
    widget_component, widget_hook, Scalar,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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
implement_props_data!(FlexBoxProps);

widget_hook! {
    use_nav_flex_box(props) {
        let direction = props.map_or_default::<FlexBoxProps, _, _>(|p| p.direction);
        let mut jump = props.read_cloned_or_default::<NavListJumpProps>();
        jump.direction = match direction {
            FlexBoxDirection::HorizontalLeftToRight => NavListDirection::HorizontalLeftToRight,
            FlexBoxDirection::HorizontalRightToLeft => NavListDirection::HorizontalRightToLeft,
            FlexBoxDirection::VerticalTopToBottom => NavListDirection::VerticalTopToBottom,
            FlexBoxDirection::VerticalBottomToTop => NavListDirection::VerticalBottomToTop,
        };
        props.write(jump);
    }
}

widget_component! {
    pub nav_flex_box(key, props, listed_slots) [
        use_nav_flex_box,
        use_nav_container_active,
        use_nav_list_active,
        use_nav_item,
    ] {
        let props = props.clone()
            .without::<NavContainerActive>()
            .without::<NavListActive>()
            .without::<NavItemActive>();

        widget!{
            (#{key} flex_box: {props} |[listed_slots]|)
        }
    }
}

widget_component! {
    pub flex_box(id, props, listed_slots) {
        let FlexBoxProps {
            direction,
            separation,
            wrap,
            transform,
        } = props.read_cloned_or_default();
        let items = listed_slots.into_iter().filter_map(|slot| {
            if let Some(props) = slot.props() {
                let layout = props.read_cloned_or_default::<FlexBoxItemLayout>();
                Some(FlexBoxItemNode {
                    slot,
                    layout,
                })
            } else {
                None
            }
        }).collect::<Vec<_>>();

        widget! {{{
            FlexBoxNode {
                id: id.to_owned(),
                props: props.clone(),
                items,
                direction,
                separation,
                wrap,
                transform,
            }
        }}}
    }
}
