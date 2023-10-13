use crate::{
    pre_hooks,
    props::Props,
    unpack_named_slots, widget,
    widget::{
        component::{
            containers::{
                anchor_box::{pivot_box, use_anchor_box, AnchorProps, PivotBoxProps},
                content_box::content_box,
                portal_box::{portal_box, use_portals_container_relative_layout},
            },
            interactive::navigation::{use_nav_container_active, use_nav_item_active, NavSignal},
        },
        context::WidgetContext,
        node::WidgetNode,
        unit::area::AreaBoxNode,
    },
    PropsData,
};
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Copy, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct TooltipState {
    #[serde(default)]
    pub show: bool,
}

#[pre_hooks(use_nav_container_active, use_nav_item_active, use_anchor_box)]
pub fn use_tooltip_box(context: &mut WidgetContext) {
    context.life_cycle.change(|context| {
        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref() {
                match msg {
                    NavSignal::Select(_) => {
                        let _ = context.state.write_with(TooltipState { show: true });
                    }
                    NavSignal::Unselect => {
                        let _ = context.state.write_with(TooltipState { show: false });
                    }
                    _ => {}
                }
            }
        }
    });
}

#[pre_hooks(use_tooltip_box)]
pub fn tooltip_box(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        idref,
        key,
        props,
        state,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => {content, tooltip});

    let TooltipState { show } = state.read_cloned_or_default();
    let anchor_state = state.read_cloned_or_default::<AnchorProps>();
    let pivot_props =
        Props::new(anchor_state).with(props.read_cloned_or_default::<PivotBoxProps>());

    let tooltip = if show {
        widget! {
            (#{"pivot"} pivot_box: {pivot_props} {
                content = (#{"portal"} portal_box {
                    content = {tooltip}
                })
            })
        }
    } else {
        widget! {()}
    };

    let content = widget! {
        (#{key} | {idref.cloned()} content_box: {props.clone()} [
            {content}
            {tooltip}
        ])
    };

    widget! {{{
        AreaBoxNode {
            id: id.to_owned(),
            slot: Box::new(content),
        }
    }}}
}

#[pre_hooks(use_portals_container_relative_layout)]
pub fn portals_tooltip_box(mut context: WidgetContext) -> WidgetNode {
    tooltip_box(context)
}
