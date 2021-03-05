use crate::{widget::WidgetIdOrRef, widget_hook, Scalar};
use serde::{Deserialize, Serialize};

fn is_false(v: &bool) -> bool {
    !*v
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct NavItemActive;
implement_props_data!(NavItemActive);

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct NavContainerActive;
implement_props_data!(NavContainerActive);

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct NavListActive;
implement_props_data!(NavListActive);

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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NavListJumpProps {
    #[serde(default)]
    pub direction: NavListDirection,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub looping: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub tabs: bool,
}
implement_props_data!(NavListJumpProps);

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum NavListDirection {
    HorizontalLeftToRight,
    HorizontalRightToLeft,
    VerticalTopToBottom,
    VerticalBottomToTop,
}

impl Default for NavListDirection {
    fn default() -> Self {
        Self::HorizontalLeftToRight
    }
}

#[derive(Debug, Clone)]
pub enum NavListJump {
    First,
    Last,
    StepLoop(isize),
    StepEscape(isize, WidgetIdOrRef),
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NavType {
    Container,
    Item,
    Button,
    TextInput,
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
    ListJump(NavListJump),
    FocusTextInput(WidgetIdOrRef),
    TextChange(NavTextChange),
    Axis(String, Scalar),
    Custom(WidgetIdOrRef, String),
}
implement_message_data!(NavSignal);

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
    pub use_nav_jump_map(life_cycle) {
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
    pub use_nav_list(props, life_cycle) {
        macro_rules! jump {
            ($context:expr, $list:expr, $jump:expr, $backward:path, $forward:path, $dir:ident) => {
                match $list.direction {
                    $backward => {
                        if $list.looping {
                            $context.signals.write(NavSignal::ListJump(
                                NavListJump::StepLoop(-1)
                            ));
                        } else {
                            $context.signals.write(NavSignal::ListJump(
                                NavListJump::StepEscape(-1, $jump.$dir.to_owned())
                            ));
                        }
                    }
                    $forward => {
                        if $list.looping {
                            $context.signals.write(NavSignal::ListJump(
                                NavListJump::StepLoop(1)
                            ));
                        } else {
                            $context.signals.write(NavSignal::ListJump(
                                NavListJump::StepEscape(1, $jump.$dir.to_owned())
                            ));
                        }
                    }
                    _ => {
                        if $jump.$dir.is_some() {
                            $context.signals.write(NavSignal::Select($jump.$dir.to_owned()));
                        }
                    }
                }
            }
        }

        macro_rules! jump_tab {
            ($context:expr, $list:expr, $jump:expr, $step:expr, $dir:ident) => {
                match $list.direction {
                    NavListDirection::HorizontalLeftToRight | NavListDirection::VerticalTopToBottom => {
                        if $list.looping {
                            $context.signals.write(NavSignal::ListJump(
                                NavListJump::StepLoop($step)
                            ));
                        } else {
                            $context.signals.write(NavSignal::ListJump(
                                NavListJump::StepEscape($step, $jump.$dir.to_owned())
                            ));
                        }
                    }
                    NavListDirection::HorizontalRightToLeft | NavListDirection::VerticalBottomToTop => {
                        if $list.looping {
                            $context.signals.write(NavSignal::ListJump(
                                NavListJump::StepLoop(- $step)
                            ));
                        } else {
                            $context.signals.write(NavSignal::ListJump(
                                NavListJump::StepEscape(- $step, $jump.$dir.to_owned())
                            ));
                        }
                    }
                }
            }
        }

        if !props.has::<NavListActive>() {
            return;
        }

        life_cycle.change(|context| {
            let list = match context.props.read::<NavListJumpProps>() {
                Ok(list) => list,
                _ => return,
            };
            let jump = context.props.read_cloned_or_default::<NavJumpMapProps>();
            for msg in context.messenger.messages {
                if let Some(msg) = msg.as_any().downcast_ref::<NavSignal>() {
                    match msg {
                        NavSignal::Up => if !list.tabs {
                            jump!(
                                context,
                                list,
                                jump,
                                NavListDirection::VerticalTopToBottom,
                                NavListDirection::VerticalBottomToTop,
                                up
                            )
                        },
                        NavSignal::Down => if !list.tabs {
                            jump!(
                                context,
                                list,
                                jump,
                                NavListDirection::VerticalBottomToTop,
                                NavListDirection::VerticalTopToBottom,
                                down
                            )
                        },
                        NavSignal::Left => if !list.tabs {
                            jump!(
                                context,
                                list,
                                jump,
                                NavListDirection::HorizontalLeftToRight,
                                NavListDirection::HorizontalRightToLeft,
                                left
                            )
                        },
                        NavSignal::Right => if !list.tabs {
                            jump!(
                                context,
                                list,
                                jump,
                                NavListDirection::HorizontalRightToLeft,
                                NavListDirection::HorizontalLeftToRight,
                                right
                            )
                        },
                        NavSignal::Prev => if list.tabs {
                            jump_tab!(
                                context,
                                list,
                                jump,
                                -1,
                                prev
                            )
                        }
                        NavSignal::Next => if list.tabs {
                            jump_tab!(
                                context,
                                list,
                                jump,
                                1,
                                next
                            )
                        }
                        _ => {}
                    }
                }
            }
        });
    }
}

widget_hook! {
    pub use_nav_list_active(props) |[use_nav_list] {
        props.write(NavListActive);
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
            context.signals.write(NavSignal::Register(NavType::Button));
        });

        life_cycle.unmount(|context| {
            context.signals.write(NavSignal::Unregister(NavType::Button));
        });
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
