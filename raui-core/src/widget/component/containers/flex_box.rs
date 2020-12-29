use crate::{
    widget,
    widget::unit::flex::{FlexBoxDirection, FlexBoxItemLayout, FlexBoxItemNode, FlexBoxNode},
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
}
implement_props_data!(FlexBoxProps, "FlexBoxProps");

widget_component! {
    pub flex_box(id, props, listed_slots) {
        let FlexBoxProps { direction, separation, wrap } = props.read_cloned_or_default();
        let items = listed_slots.into_iter().map(|slot| {
            let layout = slot
                .props()
                .expect("WidgetNode has no Props")
                .read_cloned_or_default::<FlexBoxItemLayout>();
            FlexBoxItemNode {
                slot,
                layout,
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
            }
        }}}
    }
}
