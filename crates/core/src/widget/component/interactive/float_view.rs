use crate::{
    pre_hooks, unpack_named_slots,
    widget::{
        component::{
            containers::float_box::{
                FloatBoxChange, FloatBoxChangeMessage, FloatBoxNotifyProps, FloatBoxState,
            },
            interactive::{
                button::{ButtonNotifyMessage, ButtonNotifyProps, ButtonProps, use_button},
                navigation::{
                    NavSignal, NavTrackingNotifyMessage, NavTrackingNotifyProps, use_nav_item,
                    use_nav_tracking_self,
                },
            },
        },
        context::WidgetContext,
        node::WidgetNode,
        unit::area::AreaBoxNode,
        utils::Vec2,
    },
};

#[pre_hooks(use_button, use_nav_tracking_self)]
pub fn use_float_view_control(context: &mut WidgetContext) {
    context
        .props
        .write(ButtonNotifyProps(context.id.to_owned().into()));
    context
        .props
        .write(NavTrackingNotifyProps(context.id.to_owned().into()));

    context.life_cycle.unmount(|context| {
        context.signals.write(NavSignal::Unlock);
    });

    context.life_cycle.change(|context| {
        let Some(notify) = context
            .props
            .read_cloned_or_default::<FloatBoxNotifyProps>()
            .0
            .read()
        else {
            return;
        };
        let button = context.state.read_cloned_or_default::<ButtonProps>();
        let zoom = context.props.read_cloned_or_default::<FloatBoxState>().zoom;
        let scale = if zoom > 0.0 { 1.0 / zoom } else { 1.0 };

        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<ButtonNotifyMessage>() {
                if msg.trigger_start() {
                    context.signals.write(NavSignal::Lock);
                }
                if msg.trigger_stop() {
                    context.signals.write(NavSignal::Unlock);
                }
            } else if let Some(msg) = msg.as_any().downcast_ref::<NavTrackingNotifyMessage>() {
                if button.selected && button.trigger {
                    let delta = msg.pointer_delta_ui_space();
                    context.messenger.write(
                        notify.clone(),
                        FloatBoxChangeMessage {
                            sender: context.id.to_owned(),
                            change: FloatBoxChange::RelativePosition(Vec2 {
                                x: delta.x * scale,
                                y: delta.y * scale,
                            }),
                        },
                    );
                }
            }
        }
    });
}

#[pre_hooks(use_nav_item, use_float_view_control)]
pub fn float_view_control(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        props,
        state,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => content);

    if let Some(p) = content.props_mut() {
        p.merge_from(props.clone());
        p.write(state.read_cloned_or_default::<ButtonProps>());
    }

    AreaBoxNode {
        id: id.to_owned(),
        slot: Box::new(content),
    }
    .into()
}
