use crate::{
    widget,
    widget::{
        unit::flex::{FlexBoxDirection, FlexBoxItemLayout, FlexBoxItemNode, FlexBoxNode},
        utils::Transform,
    },
    widget_component, Scalar,
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
implement_props_data!(FlexBoxProps, "FlexBoxProps");

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
