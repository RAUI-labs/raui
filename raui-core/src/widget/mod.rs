pub mod component;
pub mod context;
pub mod node;
pub mod unit;
pub mod utils;

use crate::{
    messenger::MessageSender,
    signals::SignalSender,
    state::StateData,
    widget::{context::WidgetContext, node::WidgetNode},
};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

#[derive(Default, Hash, Eq, PartialEq, Clone)]
pub struct WidgetId {
    type_name: String,
    path: Vec<String>,
}

impl WidgetId {
    pub fn new(type_name: String, path: Vec<String>) -> Self {
        Self { type_name, path }
    }

    pub fn depth(&self) -> usize {
        self.path.len()
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    pub fn key(&self) -> Option<&str> {
        self.path.last().map(|v| v.as_str())
    }

    pub fn hashed_value(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl ToString for WidgetId {
    fn to_string(&self) -> String {
        let count =
            self.type_name.len() + 1 + self.path.iter().map(|part| 1 + part.len()).sum::<usize>();
        let mut result = String::with_capacity(count);
        result.push_str(&self.type_name);
        result.push(':');
        for part in &self.path {
            result.push('/');
            result.push_str(part);
        }
        result
    }
}

impl std::fmt::Debug for WidgetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string())
    }
}

pub type FnWidget = fn(WidgetContext) -> WidgetNode;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WidgetPhase {
    Mount,
    Update,
}

pub type WidgetUnmountClosure = dyn FnMut(&WidgetId, &StateData, &MessageSender, &SignalSender);

#[derive(Default)]
pub struct WidgetUnmounter(Option<Box<WidgetUnmountClosure>>);

impl WidgetUnmounter {
    pub fn listen<F>(&mut self, f: F)
    where
        F: 'static + FnMut(&WidgetId, &StateData, &MessageSender, &SignalSender),
    {
        self.0 = Some(Box::new(f));
    }

    pub fn into_inner(self) -> Option<Box<WidgetUnmountClosure>> {
        self.0
    }
}

#[macro_export]
macro_rules! widget {
    {()} => ($crate::widget::node::WidgetNode::None);
    {[]} => ($crate::widget::node::WidgetNode::Unit(Default::default()));
    ({{$expr:expr}}) => {
        $crate::widget::node::WidgetNode::Unit($crate::widget::unit::WidgetUnit::from($expr))
    };
    ({$expr:expr}) => {
        $crate::widget::node::WidgetNode::from($expr)
    };
    {
        (
            $(
                #{ $key:expr }
            )?
            $type_id:path
            $(
                : {$props:expr}
            )?
            $(
                {
                    $($named_slot_name:ident = $named_slot_widget:tt)+
                }
            )?
            $(
                [
                    $($listed_slot_widget:tt)+
                ]
            )?
        )
    } => {
        {
            #[allow(unused_assignments)]
            #[allow(unused_mut)]
            let mut key = None;
            $(
                key = Some($key.to_string());
            )?
            let processor = $type_id;
            let type_name = stringify!($type_id).to_owned();
            #[allow(unused_assignments)]
            #[allow(unused_mut)]
            let mut props = $crate::props::Props::default();
            $(
                props = $crate::props::Props::new($props);
            )?
            #[allow(unused_mut)]
            let mut named_slots = std::collections::HashMap::new();
            $(
                $(
                    let widget = widget_wrap!{$named_slot_widget};
                    if widget.is_some() {
                        let name = stringify!($named_slot_name).to_owned();
                        named_slots.insert(name, widget);
                    }
                )+
            )?
            #[allow(unused_mut)]
            let mut listed_slots = vec![];
            $(
                $(
                    let widget = widget_wrap!{$listed_slot_widget};
                    if widget.is_some() {
                        listed_slots.push(widget);
                    }
                )*
            )?
            let component = $crate::widget::component::WidgetComponent {
                processor,
                type_name,
                key,
                props,
                named_slots,
                listed_slots,
            };
            $crate::widget::node::WidgetNode::Component(component)
        }
    };
}

#[macro_export]
macro_rules! widget_wrap {
    ({$expr:expr}) => {
        $crate::widget::node::WidgetNode::from($expr)
    };
    ($tree:tt) => {
        widget!($tree)
    };
}

#[macro_export]
macro_rules! destruct {
    {$type_id:path { $($prop:ident),+ } ($value:expr) => $code:block} => {
        #[allow(unused_variables)]
        match $value {
            $type_id { $( $prop ),+ , .. } => $code
        }
    };
}

#[macro_export]
macro_rules! unpack_context {
    ($value:expr => { $($prop:ident),+ }) => {
        #[allow(unused_variables)]
        #[allow(unused_mut)]
        let ( $( mut $prop ),+ ) = match $value {
            $crate::widget::context::WidgetContext { $( $prop ),+ , .. } => ( $( $prop ),+ ),
        };
    };
}

#[macro_export]
macro_rules! unpack_named_slots {
    ($map:expr => { $($name:ident),+ }) => {
        #[allow(unused_variables)]
        let ( $( $name ),+ ) = {
            let mut map = $map;
            (
                $(
                    {
                        let name = stringify!($name);
                        match map.remove(name) {
                            Some(widget) => widget,
                            None => $crate::widget::node::WidgetNode::None,
                        }
                    }
                ),+
            )
        };
    };
}

#[macro_export]
macro_rules! widget_component {
    {$name:ident $( ( $( $param:ident ),+ ) )? $code:block} => {
        #[allow(unused_variables)]
        #[allow(unused_mut)]
        fn $name(
            context: $crate::widget::context::WidgetContext
        ) -> $crate::widget::node::WidgetNode {
            $(
                unpack_context!(context => { $( $param ),+ });
            )?
            $code
        }
    };
}
