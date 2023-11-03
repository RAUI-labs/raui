use crate::{
    post_hooks, pre_hooks, unpack_named_slots,
    widget::{
        component::containers::portal_box::PortalsContainer, context::WidgetContext,
        node::WidgetNode, unit::area::AreaBoxNode, utils::Vec2, WidgetId, WidgetIdOrRef,
    },
    MessageData, PropsData, Scalar,
};
use serde::{Deserialize, Serialize};

#[derive(PropsData, Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct NavItemActive;

#[derive(PropsData, Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct NavTrackingActive(#[serde(default)] pub WidgetIdOrRef);

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct NavTrackingNotifyProps(
    #[serde(default)]
    #[serde(skip_serializing_if = "WidgetIdOrRef::is_none")]
    pub WidgetIdOrRef,
);

#[derive(PropsData, Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct NavTrackingProps(#[serde(default)] pub Vec2);

#[derive(MessageData, Debug, Default, Clone)]
#[message_data(crate::messenger::MessageData)]
pub struct NavTrackingNotifyMessage {
    pub sender: WidgetId,
    pub state: NavTrackingProps,
    pub prev: NavTrackingProps,
}

impl NavTrackingNotifyMessage {
    pub fn pointer_delta(&self) -> Vec2 {
        Vec2 {
            x: self.state.0.x - self.prev.0.x,
            y: self.state.0.y - self.prev.0.y,
        }
    }

    pub fn pointer_moved(&self) -> bool {
        (self.state.0.x - self.prev.0.x) + (self.state.0.y - self.prev.0.y) > 1.0e-6
    }
}

#[derive(PropsData, Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct NavLockingActive;

#[derive(PropsData, Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct NavContainerActive;

#[derive(PropsData, Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct NavJumpActive(#[serde(default)] pub NavJumpMode);

#[derive(PropsData, Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct NavJumpLooped;

#[derive(Debug, Clone, PartialEq)]
pub enum NavType {
    Container,
    Item,
    Button,
    TextInput,
    ScrollView,
    ScrollViewContent,
    /// (tracked widget)
    Tracking(WidgetIdOrRef),
}

#[derive(MessageData, Debug, Default, Clone)]
#[message_data(crate::messenger::MessageData)]
pub enum NavSignal {
    #[default]
    None,
    Register(NavType),
    Unregister(NavType),
    Select(WidgetIdOrRef),
    Unselect,
    Lock,
    Unlock,
    Accept(bool),
    Context(bool),
    Cancel(bool),
    Up,
    Down,
    Left,
    Right,
    Prev,
    Next,
    Jump(NavJump),
    FocusTextInput(WidgetIdOrRef),
    TextChange(NavTextChange),
    Axis(String, Scalar),
    Custom(WidgetIdOrRef, String),
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NavJumpMode {
    #[default]
    Direction,
    StepHorizontal,
    StepVertical,
    StepPages,
}

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct NavJumpMapProps {
    #[serde(default)]
    #[serde(skip_serializing_if = "WidgetIdOrRef::is_none")]
    pub up: WidgetIdOrRef,
    #[serde(default)]
    #[serde(skip_serializing_if = "WidgetIdOrRef::is_none")]
    pub down: WidgetIdOrRef,
    #[serde(default)]
    #[serde(skip_serializing_if = "WidgetIdOrRef::is_none")]
    pub left: WidgetIdOrRef,
    #[serde(default)]
    #[serde(skip_serializing_if = "WidgetIdOrRef::is_none")]
    pub right: WidgetIdOrRef,
    #[serde(default)]
    #[serde(skip_serializing_if = "WidgetIdOrRef::is_none")]
    pub prev: WidgetIdOrRef,
    #[serde(default)]
    #[serde(skip_serializing_if = "WidgetIdOrRef::is_none")]
    pub next: WidgetIdOrRef,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NavDirection {
    #[default]
    None,
    Up,
    Down,
    Left,
    Right,
    Prev,
    Next,
}

#[derive(Debug, Clone)]
pub enum NavJump {
    First,
    Last,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    MiddleCenter,
    Loop(NavDirection),
    Escape(NavDirection, WidgetIdOrRef),
    Scroll(NavScroll),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NavTextChange {
    InsertCharacter(char),
    MoveCursorLeft,
    MoveCursorRight,
    MoveCursorStart,
    MoveCursorEnd,
    DeleteLeft,
    DeleteRight,
    NewLine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NavScroll {
    /// (factor location, relative)
    Factor(Vec2, bool),
    /// (scroll view id or ref, factor location, relative)
    DirectFactor(WidgetIdOrRef, Vec2, bool),
    /// (local space units location, relative)
    Units(Vec2, bool),
    /// (scroll view id or ref, local space units location, relative)
    DirectUnits(WidgetIdOrRef, Vec2, bool),
    /// (id or ref, widget local space anchor point)
    Widget(WidgetIdOrRef, Vec2),
    /// (factor, content to container ratio, relative)
    Change(Vec2, Vec2, bool),
}

pub fn use_nav_container(context: &mut WidgetContext) {
    context.life_cycle.mount(|context| {
        if context.props.has::<NavContainerActive>() {
            context
                .signals
                .write(NavSignal::Register(NavType::Container));
        }
    });

    context.life_cycle.unmount(|context| {
        context
            .signals
            .write(NavSignal::Unregister(NavType::Container));
    });
}

#[post_hooks(use_nav_container)]
pub fn use_nav_container_active(context: &mut WidgetContext) {
    context.props.write(NavContainerActive);
}

pub fn use_nav_jump_map(context: &mut WidgetContext) {
    if !context.props.has::<NavJumpActive>() {
        return;
    }

    context.life_cycle.change(|context| {
        let jump = match context.props.read::<NavJumpMapProps>() {
            Ok(jump) => jump,
            _ => return,
        };
        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref() {
                match msg {
                    NavSignal::Up => {
                        if jump.up.is_some() {
                            context.signals.write(NavSignal::Select(jump.up.to_owned()));
                        }
                    }
                    NavSignal::Down => {
                        if jump.down.is_some() {
                            context
                                .signals
                                .write(NavSignal::Select(jump.down.to_owned()));
                        }
                    }
                    NavSignal::Left => {
                        if jump.left.is_some() {
                            context
                                .signals
                                .write(NavSignal::Select(jump.left.to_owned()));
                        }
                    }
                    NavSignal::Right => {
                        if jump.right.is_some() {
                            context
                                .signals
                                .write(NavSignal::Select(jump.right.to_owned()));
                        }
                    }
                    NavSignal::Prev => {
                        if jump.prev.is_some() {
                            context
                                .signals
                                .write(NavSignal::Select(jump.prev.to_owned()));
                        }
                    }
                    NavSignal::Next => {
                        if jump.next.is_some() {
                            context
                                .signals
                                .write(NavSignal::Select(jump.next.to_owned()));
                        }
                    }
                    _ => {}
                }
            }
        }
    });
}

pub fn use_nav_jump(context: &mut WidgetContext) {
    context.life_cycle.change(|context| {
        let mode = match context.props.read::<NavJumpActive>() {
            Ok(data) => data.0,
            Err(_) => return,
        };
        let looped = context.props.has::<NavJumpLooped>();
        let jump = context.props.read_cloned_or_default::<NavJumpMapProps>();
        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref() {
                match (mode, msg) {
                    (NavJumpMode::Direction, NavSignal::Up) => {
                        if looped {
                            context
                                .signals
                                .write(NavSignal::Jump(NavJump::Loop(NavDirection::Up)));
                        } else {
                            context.signals.write(NavSignal::Jump(NavJump::Escape(
                                NavDirection::Up,
                                jump.up.to_owned(),
                            )));
                        }
                    }
                    (NavJumpMode::Direction, NavSignal::Down) => {
                        if looped {
                            context
                                .signals
                                .write(NavSignal::Jump(NavJump::Loop(NavDirection::Down)));
                        } else {
                            context.signals.write(NavSignal::Jump(NavJump::Escape(
                                NavDirection::Down,
                                jump.down.to_owned(),
                            )));
                        }
                    }
                    (NavJumpMode::Direction, NavSignal::Left) => {
                        if looped {
                            context
                                .signals
                                .write(NavSignal::Jump(NavJump::Loop(NavDirection::Left)));
                        } else {
                            context.signals.write(NavSignal::Jump(NavJump::Escape(
                                NavDirection::Left,
                                jump.left.to_owned(),
                            )));
                        }
                    }
                    (NavJumpMode::Direction, NavSignal::Right) => {
                        if looped {
                            context
                                .signals
                                .write(NavSignal::Jump(NavJump::Loop(NavDirection::Right)));
                        } else {
                            context.signals.write(NavSignal::Jump(NavJump::Escape(
                                NavDirection::Right,
                                jump.right.to_owned(),
                            )));
                        }
                    }
                    (NavJumpMode::StepHorizontal, NavSignal::Left) => {
                        if looped {
                            context
                                .signals
                                .write(NavSignal::Jump(NavJump::Loop(NavDirection::Prev)));
                        } else {
                            context.signals.write(NavSignal::Jump(NavJump::Escape(
                                NavDirection::Prev,
                                jump.left.to_owned(),
                            )));
                        }
                    }
                    (NavJumpMode::StepHorizontal, NavSignal::Right) => {
                        if looped {
                            context
                                .signals
                                .write(NavSignal::Jump(NavJump::Loop(NavDirection::Next)));
                        } else {
                            context.signals.write(NavSignal::Jump(NavJump::Escape(
                                NavDirection::Next,
                                jump.right.to_owned(),
                            )));
                        }
                    }
                    (NavJumpMode::StepVertical, NavSignal::Up) => {
                        if looped {
                            context
                                .signals
                                .write(NavSignal::Jump(NavJump::Loop(NavDirection::Prev)));
                        } else {
                            context.signals.write(NavSignal::Jump(NavJump::Escape(
                                NavDirection::Prev,
                                jump.up.to_owned(),
                            )));
                        }
                    }
                    (NavJumpMode::StepVertical, NavSignal::Down) => {
                        if looped {
                            context
                                .signals
                                .write(NavSignal::Jump(NavJump::Loop(NavDirection::Next)));
                        } else {
                            context.signals.write(NavSignal::Jump(NavJump::Escape(
                                NavDirection::Next,
                                jump.down.to_owned(),
                            )));
                        }
                    }
                    (NavJumpMode::StepPages, NavSignal::Prev) => {
                        if looped {
                            context
                                .signals
                                .write(NavSignal::Jump(NavJump::Loop(NavDirection::Prev)));
                        } else {
                            context.signals.write(NavSignal::Jump(NavJump::Escape(
                                NavDirection::Prev,
                                jump.prev.to_owned(),
                            )));
                        }
                    }
                    (NavJumpMode::StepPages, NavSignal::Next) => {
                        if looped {
                            context
                                .signals
                                .write(NavSignal::Jump(NavJump::Loop(NavDirection::Next)));
                        } else {
                            context.signals.write(NavSignal::Jump(NavJump::Escape(
                                NavDirection::Next,
                                jump.next.to_owned(),
                            )));
                        }
                    }
                    _ => {}
                }
            }
        }
    });
}

#[post_hooks(use_nav_jump)]
pub fn use_nav_jump_direction_active(context: &mut WidgetContext) {
    context.props.write(NavJumpActive(NavJumpMode::Direction));
}

#[post_hooks(use_nav_jump)]
pub fn use_nav_jump_horizontal_step_active(context: &mut WidgetContext) {
    context
        .props
        .write(NavJumpActive(NavJumpMode::StepHorizontal));
}

#[post_hooks(use_nav_jump)]
pub fn use_nav_jump_vertical_step_active(context: &mut WidgetContext) {
    context
        .props
        .write(NavJumpActive(NavJumpMode::StepVertical));
}

#[post_hooks(use_nav_jump)]
pub fn use_nav_jump_step_pages_active(context: &mut WidgetContext) {
    context.props.write(NavJumpActive(NavJumpMode::StepPages));
}

pub fn use_nav_item(context: &mut WidgetContext) {
    context.life_cycle.mount(|context| {
        if context.props.has::<NavItemActive>() {
            context.signals.write(NavSignal::Register(NavType::Item));
        }
    });

    context.life_cycle.unmount(|context| {
        context.signals.write(NavSignal::Unregister(NavType::Item));
    });
}

#[post_hooks(use_nav_item)]
pub fn use_nav_item_active(context: &mut WidgetContext) {
    context.props.write(NavItemActive);
}

pub fn use_nav_button(context: &mut WidgetContext) {
    context.life_cycle.mount(|context| {
        context.signals.write(NavSignal::Register(NavType::Button));
    });

    context.life_cycle.unmount(|context| {
        context
            .signals
            .write(NavSignal::Unregister(NavType::Button));
    });
}

pub fn use_nav_tracking(context: &mut WidgetContext) {
    context.life_cycle.mount(|context| {
        if let Ok(tracking) = context.props.read::<NavTrackingActive>() {
            context
                .signals
                .write(NavSignal::Register(NavType::Tracking(tracking.0.clone())));
            let _ = context.state.write_with(NavTrackingProps::default());
        }
    });

    context.life_cycle.unmount(|context| {
        context
            .signals
            .write(NavSignal::Unregister(NavType::Tracking(Default::default())));
    });

    context.life_cycle.change(|context| {
        if let Ok(tracking) = context.props.read::<NavTrackingActive>() {
            if !context.state.has::<NavTrackingProps>() {
                context
                    .signals
                    .write(NavSignal::Register(NavType::Tracking(tracking.0.clone())));
                let _ = context.state.write_with(NavTrackingProps::default());
            }
            let mut dirty = false;
            let mut data = context.state.read_cloned_or_default::<NavTrackingProps>();
            let prev = data;
            for msg in context.messenger.messages {
                if let Some(NavSignal::Axis(axis, value)) = msg.as_any().downcast_ref::<NavSignal>()
                {
                    if axis == "pointer-x" {
                        data.0.x = *value;
                        dirty = true;
                    } else if axis == "pointer-y" {
                        data.0.y = *value;
                        dirty = true;
                    }
                }
            }
            if dirty {
                if let Ok(NavTrackingNotifyProps(notify)) = context.props.read() {
                    if let Some(to) = notify.read() {
                        context.messenger.write(
                            to,
                            NavTrackingNotifyMessage {
                                sender: context.id.to_owned(),
                                state: data.to_owned(),
                                prev,
                            },
                        );
                    }
                }
                let _ = context.state.write_with(data);
            }
        } else if context.state.has::<NavTrackingProps>() {
            context
                .signals
                .write(NavSignal::Unregister(NavType::Tracking(Default::default())));
            let _ = context.state.write_without::<NavTrackingProps>();
        }
    });
}

#[pre_hooks(use_nav_tracking)]
pub fn use_nav_tracking_self(context: &mut WidgetContext) {
    context
        .props
        .write(NavTrackingActive(context.id.to_owned().into()));
}

#[pre_hooks(use_nav_tracking)]
pub fn use_nav_tracking_active_portals_container(context: &mut WidgetContext) {
    if let Ok(data) = context.shared_props.read::<PortalsContainer>() {
        context
            .props
            .write(NavTrackingActive(data.0.to_owned().into()));
    }
}

pub fn use_nav_tracking_notified_state(context: &mut WidgetContext) {
    context.life_cycle.change(|context| {
        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<NavTrackingNotifyMessage>() {
                let _ = context.state.write_with(msg.state);
            }
        }
    });
}

pub fn use_nav_locking(context: &mut WidgetContext) {
    context.life_cycle.mount(|context| {
        if context.props.has::<NavLockingActive>() {
            context.signals.write(NavSignal::Lock);
            let _ = context.state.write_with(NavLockingActive);
        }
    });

    context.life_cycle.unmount(|context| {
        context.signals.write(NavSignal::Unlock);
    });

    context.life_cycle.change(|context| {
        if context.props.has::<NavLockingActive>() {
            if !context.state.has::<NavLockingActive>() {
                context.signals.write(NavSignal::Lock);
                let _ = context.state.write_with(NavLockingActive);
            }
        } else if context.state.has::<NavLockingActive>()
            && !context.props.has::<NavLockingActive>()
        {
            context.signals.write(NavSignal::Unlock);
            let _ = context.state.write_without::<NavLockingActive>();
        }
    });
}

pub fn use_nav_text_input(context: &mut WidgetContext) {
    context.life_cycle.mount(|context| {
        context
            .signals
            .write(NavSignal::Register(NavType::TextInput));
    });

    context.life_cycle.unmount(|context| {
        context
            .signals
            .write(NavSignal::Unregister(NavType::TextInput));
    });
}

pub fn use_nav_scroll_view(context: &mut WidgetContext) {
    context.life_cycle.mount(|context| {
        context
            .signals
            .write(NavSignal::Register(NavType::ScrollView));
    });

    context.life_cycle.unmount(|context| {
        context
            .signals
            .write(NavSignal::Unregister(NavType::ScrollView));
    });
}

pub fn use_nav_scroll_view_content(context: &mut WidgetContext) {
    context.life_cycle.mount(|context| {
        context
            .signals
            .write(NavSignal::Register(NavType::ScrollViewContent));
    });

    context.life_cycle.unmount(|context| {
        context
            .signals
            .write(NavSignal::Unregister(NavType::ScrollViewContent));
    });
}

#[pre_hooks(use_nav_button)]
pub fn navigation_barrier(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id, named_slots, ..
    } = context;
    unpack_named_slots!(named_slots => content);

    AreaBoxNode {
        id: id.to_owned(),
        slot: Box::new(content),
    }
    .into()
}

#[pre_hooks(use_nav_tracking)]
pub fn tracking(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id, named_slots, ..
    } = context;
    unpack_named_slots!(named_slots => content);

    AreaBoxNode {
        id: id.to_owned(),
        slot: Box::new(content),
    }
    .into()
}

#[pre_hooks(use_nav_tracking_self)]
pub fn self_tracking(mut context: WidgetContext) -> WidgetNode {
    tracking(context)
}
