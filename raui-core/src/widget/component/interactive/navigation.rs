use crate::{
    widget::{utils::Vec2, WidgetIdOrRef},
    widget_hook, Scalar,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavItemActive;
implement_props_data!(NavItemActive);

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavButtonTrackingActive;
implement_props_data!(NavButtonTrackingActive);

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavContainerActive;
implement_props_data!(NavContainerActive);

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavJumpActive(#[serde(default)] pub NavJumpMode);
implement_props_data!(NavJumpActive);

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavJumpLooped;
implement_props_data!(NavJumpLooped);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NavType {
    Container,
    Item,
    /// (track pointer)
    Button(bool),
    TextInput,
    ScrollView,
    ScrollViewContent,
}

#[derive(Debug, Clone)]
pub enum NavSignal {
    None,
    Register(NavType),
    Unregister(NavType),
    Select(WidgetIdOrRef),
    Unselect,
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
implement_message_data!(NavSignal);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NavJumpMode {
    Direction,
    StepHorizontal,
    StepVertical,
    StepPages,
}

impl Default for NavJumpMode {
    fn default() -> Self {
        Self::Direction
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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
implement_props_data!(NavJumpMapProps);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NavDirection {
    None,
    Up,
    Down,
    Left,
    Right,
    Prev,
    Next,
}

impl Default for NavDirection {
    fn default() -> Self {
        Self::None
    }
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

impl Default for NavSignal {
    fn default() -> Self {
        Self::None
    }
}

widget_hook! {
    pub use_nav_container(life_cycle) {
        life_cycle.mount(|context| {
            if context.props.has::<NavContainerActive>() {
                context.signals.write(NavSignal::Register(NavType::Container));
            }
        });

        life_cycle.unmount(|context| {
            context.signals.write(NavSignal::Unregister(NavType::Container));
        });
    }
}

widget_hook! {
    pub use_nav_container_active(props) |[use_nav_container] {
        props.write(NavContainerActive);
    }
}

widget_hook! {
    pub use_nav_jump_map(props, life_cycle) {
        if !props.has::<NavJumpActive>() {
            return;
        }

        life_cycle.change(|context| {
            let jump = match context.props.read::<NavJumpMapProps>() {
                Ok(jump) => jump,
                _ => return,
            };
            for msg in context.messenger.messages {
                if let Some(msg) = msg.as_any().downcast_ref::<NavSignal>() {
                    match msg {
                        NavSignal::Up => {
                            if jump.up.is_some() {
                                context.signals.write(NavSignal::Select(jump.up.to_owned()));
                            }
                        }
                        NavSignal::Down => {
                            if jump.down.is_some() {
                                context.signals.write(NavSignal::Select(jump.down.to_owned()));
                            }
                        }
                        NavSignal::Left => {
                            if jump.left.is_some() {
                                context.signals.write(NavSignal::Select(jump.left.to_owned()));
                            }
                        }
                        NavSignal::Right => {
                            if jump.right.is_some() {
                                context.signals.write(NavSignal::Select(jump.right.to_owned()));
                            }
                        }
                        NavSignal::Prev => {
                            if jump.prev.is_some() {
                                context.signals.write(NavSignal::Select(jump.prev.to_owned()));
                            }
                        }
                        NavSignal::Next => {
                            if jump.next.is_some() {
                                context.signals.write(NavSignal::Select(jump.next.to_owned()));
                            }
                        }
                        _ => {}
                    }
                }
            }
        });
    }
}

widget_hook! {
    pub use_nav_jump(life_cycle) {
        life_cycle.change(|context| {
            let mode = match context.props.read::<NavJumpActive>() {
                Ok(data) => data.0,
                Err(_) => return,
            };
            let looped = context.props.has::<NavJumpLooped>();
            let jump = context.props.read_cloned_or_default::<NavJumpMapProps>();
            for msg in context.messenger.messages {
                if let Some(msg) = msg.as_any().downcast_ref::<NavSignal>() {
                    match (mode, msg) {
                        (NavJumpMode::Direction, NavSignal::Up) => {
                            if looped {
                                context.signals.write(NavSignal::Jump(
                                    NavJump::Loop(NavDirection::Up)
                                ));
                            } else {
                                context.signals.write(NavSignal::Jump(
                                    NavJump::Escape(NavDirection::Up, jump.up.to_owned())
                                ));
                            }
                        }
                        (NavJumpMode::Direction, NavSignal::Down) => {
                            if looped {
                                context.signals.write(NavSignal::Jump(
                                    NavJump::Loop(NavDirection::Down)
                                ));
                            } else {
                                context.signals.write(NavSignal::Jump(
                                    NavJump::Escape(NavDirection::Down, jump.down.to_owned())
                                ));
                            }
                        }
                        (NavJumpMode::Direction, NavSignal::Left) => {
                            if looped {
                                context.signals.write(NavSignal::Jump(
                                    NavJump::Loop(NavDirection::Left)
                                ));
                            } else {
                                context.signals.write(NavSignal::Jump(
                                    NavJump::Escape(NavDirection::Left, jump.left.to_owned())
                                ));
                            }
                        }
                        (NavJumpMode::Direction, NavSignal::Right) => {
                            if looped {
                                context.signals.write(NavSignal::Jump(
                                    NavJump::Loop(NavDirection::Right)
                                ));
                            } else {
                                context.signals.write(NavSignal::Jump(
                                    NavJump::Escape(NavDirection::Right, jump.right.to_owned())
                                ));
                            }
                        }
                        (NavJumpMode::StepHorizontal, NavSignal::Left) => {
                            if looped {
                                context.signals.write(NavSignal::Jump(
                                    NavJump::Loop(NavDirection::Prev)
                                ));
                            } else {
                                context.signals.write(NavSignal::Jump(
                                    NavJump::Escape(NavDirection::Prev, jump.left.to_owned())
                                ));
                            }
                        }
                        (NavJumpMode::StepHorizontal, NavSignal::Right) => {
                            if looped {
                                context.signals.write(NavSignal::Jump(
                                    NavJump::Loop(NavDirection::Next)
                                ));
                            } else {
                                context.signals.write(NavSignal::Jump(
                                    NavJump::Escape(NavDirection::Next, jump.right.to_owned())
                                ));
                            }
                        }
                        (NavJumpMode::StepVertical, NavSignal::Up) => {
                            if looped {
                                context.signals.write(NavSignal::Jump(
                                    NavJump::Loop(NavDirection::Prev)
                                ));
                            } else {
                                context.signals.write(NavSignal::Jump(
                                    NavJump::Escape(NavDirection::Prev, jump.up.to_owned())
                                ));
                            }
                        }
                        (NavJumpMode::StepVertical, NavSignal::Down) => {
                            if looped {
                                context.signals.write(NavSignal::Jump(
                                    NavJump::Loop(NavDirection::Next)
                                ));
                            } else {
                                context.signals.write(NavSignal::Jump(
                                    NavJump::Escape(NavDirection::Next, jump.down.to_owned())
                                ));
                            }
                        }
                        (NavJumpMode::StepPages, NavSignal::Prev) => {
                            if looped {
                                context.signals.write(NavSignal::Jump(
                                    NavJump::Loop(NavDirection::Prev)
                                ));
                            } else {
                                context.signals.write(NavSignal::Jump(
                                    NavJump::Escape(NavDirection::Prev, jump.prev.to_owned())
                                ));
                            }
                        }
                        (NavJumpMode::StepPages, NavSignal::Next) => {
                            if looped {
                                context.signals.write(NavSignal::Jump(
                                    NavJump::Loop(NavDirection::Next)
                                ));
                            } else {
                                context.signals.write(NavSignal::Jump(
                                    NavJump::Escape(NavDirection::Next, jump.next.to_owned())
                                ));
                            }
                        }
                        _ => {}
                    }
                }
            }
        });
    }
}

widget_hook! {
    pub use_nav_jump_direction_active(props) |[use_nav_jump] {
        props.write(NavJumpActive(NavJumpMode::Direction));
    }
}

widget_hook! {
    pub use_nav_jump_horizontal_step_active(props) |[use_nav_jump] {
        props.write(NavJumpActive(NavJumpMode::StepHorizontal));
    }
}

widget_hook! {
    pub use_nav_jump_vertical_step_active(props) |[use_nav_jump] {
        props.write(NavJumpActive(NavJumpMode::StepVertical));
    }
}

widget_hook! {
    pub use_nav_jump_step_pages_active(props) |[use_nav_jump] {
        props.write(NavJumpActive(NavJumpMode::StepPages));
    }
}

widget_hook! {
    pub use_nav_item(life_cycle) {
        life_cycle.mount(|context| {
            if context.props.has::<NavItemActive>() {
                context.signals.write(NavSignal::Register(NavType::Item));
            }
        });

        life_cycle.unmount(|context| {
            context.signals.write(NavSignal::Unregister(NavType::Item));
        });
    }
}

widget_hook! {
    pub use_nav_item_active(props) |[use_nav_item] {
        props.write(NavItemActive);
    }
}

widget_hook! {
    pub use_nav_button(life_cycle) {
        life_cycle.mount(|context| {
            let tracked = context.props.has::<NavButtonTrackingActive>();
            context.signals.write(NavSignal::Register(NavType::Button(tracked)));
        });

        life_cycle.unmount(|context| {
            context.signals.write(NavSignal::Unregister(NavType::Button(false)));
        });
    }
}

widget_hook! {
    pub use_nav_button_tracking_active(props) |[use_nav_button] {
        props.write(NavButtonTrackingActive);
    }
}

widget_hook! {
    pub use_nav_text_input(life_cycle) {
        life_cycle.mount(|context| {
            context.signals.write(NavSignal::Register(NavType::TextInput));
        });

        life_cycle.unmount(|context| {
            context.signals.write(NavSignal::Unregister(NavType::TextInput));
        });
    }
}

widget_hook! {
    pub use_nav_scroll_view(life_cycle) {
        life_cycle.mount(|context| {
            context.signals.write(NavSignal::Register(NavType::ScrollView));
        });

        life_cycle.unmount(|context| {
            context.signals.write(NavSignal::Unregister(NavType::ScrollView));
        });
    }
}

widget_hook! {
    pub use_nav_scroll_view_content(life_cycle) {
        life_cycle.mount(|context| {
            context.signals.write(NavSignal::Register(NavType::ScrollViewContent));
        });

        life_cycle.unmount(|context| {
            context.signals.write(NavSignal::Unregister(NavType::ScrollViewContent));
        });
    }
}
