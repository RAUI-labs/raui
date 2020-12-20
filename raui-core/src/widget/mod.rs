pub mod component;
pub mod context;
pub mod node;
pub mod unit;
pub mod utils;

use crate::{
    messenger::{MessageSender, Messenger},
    props::Props,
    signals::SignalSender,
    state::{State, StateData},
    widget::{context::WidgetContext, node::WidgetNode},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    ops::Deref,
    str::FromStr,
};

#[derive(Default, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct WidgetId {
    id: String,
    type_name_len: u8,
    key_len: u8,
    depth: usize,
}

impl WidgetId {
    pub fn new(type_name: String, path: Vec<String>) -> Self {
        if type_name.len() >= 256 {
            panic!(
                "WidgetId `type_name` (\"{}\") cannot be longer than 255 characters!",
                type_name
            );
        }
        let type_name_len = type_name.len() as u8;
        let key_len = path.last().map(|p| p.len()).unwrap_or_default() as u8;
        let depth = path.len();
        let count = type_name.len() + 1 + path.iter().map(|part| 1 + part.len()).sum::<usize>();
        let mut id = String::with_capacity(count);
        id.push_str(&type_name);
        id.push(':');
        for (i, part) in path.into_iter().enumerate() {
            if part.len() >= 256 {
                panic!(
                    "WidgetId `path[{}]` (\"{}\") cannot be longer than 255 characters!",
                    i, part
                );
            }
            id.push('/');
            id.push_str(&part);
        }
        Self {
            id,
            type_name_len,
            key_len,
            depth,
        }
    }

    #[inline]
    pub fn depth(&self) -> usize {
        self.depth
    }

    #[inline]
    pub fn type_name(&self) -> &str {
        &self.id.as_str()[0..self.type_name_len as usize]
    }

    #[inline]
    pub fn key(&self) -> &str {
        &self.id[(self.id.len() - self.key_len as usize)..]
    }

    #[inline]
    pub fn parts(&self) -> impl Iterator<Item = &str> {
        self.id[(self.type_name_len as usize + 2)..].split('/')
    }

    pub fn hashed_value(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl Deref for WidgetId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl AsRef<str> for WidgetId {
    fn as_ref(&self) -> &str {
        &self.id
    }
}

impl FromStr for WidgetId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(index) = s.find(':') {
            let type_name = s[..index].to_owned();
            let rest = &s[(index + 2)..];
            let path = rest.split('/').map(|p| p.to_owned()).collect::<Vec<_>>();
            Ok(Self::new(type_name, path))
        } else {
            Err(())
        }
    }
}

impl std::fmt::Debug for WidgetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

pub type FnWidget = fn(WidgetContext) -> WidgetNode;

pub type WidgetMountOrChangeClosure =
    dyn FnMut(&WidgetId, &Props, &State, &Messenger, &SignalSender);
pub type WidgetUnmountClosure = dyn FnMut(&WidgetId, &StateData, &MessageSender, &SignalSender);

#[derive(Default)]
pub struct WidgetLifeCycle {
    mount: Vec<Box<WidgetMountOrChangeClosure>>,
    change: Vec<Box<WidgetMountOrChangeClosure>>,
    unmount: Vec<Box<WidgetUnmountClosure>>,
}

impl WidgetLifeCycle {
    pub fn mount<F>(&mut self, f: F)
    where
        F: 'static + FnMut(&WidgetId, &Props, &State, &Messenger, &SignalSender),
    {
        self.mount.push(Box::new(f));
    }

    pub fn change<F>(&mut self, f: F)
    where
        F: 'static + FnMut(&WidgetId, &Props, &State, &Messenger, &SignalSender),
    {
        self.change.push(Box::new(f));
    }

    pub fn unmount<F>(&mut self, f: F)
    where
        F: 'static + FnMut(&WidgetId, &StateData, &MessageSender, &SignalSender),
    {
        self.unmount.push(Box::new(f));
    }

    pub fn unwrap(
        self,
    ) -> (
        Vec<Box<WidgetMountOrChangeClosure>>,
        Vec<Box<WidgetMountOrChangeClosure>>,
        Vec<Box<WidgetUnmountClosure>>,
    ) {
        let Self {
            mount,
            change,
            unmount,
        } = self;
        (mount, change, unmount)
    }
}

#[macro_export]
macro_rules! widget {
    {()} => ($crate::widget::node::WidgetNode::None);
    {[]} => ($crate::widget::node::WidgetNode::Unit(Default::default()));
    ({{$expr:expr}}) => {
        $crate::widget::node::WidgetNode::Unit($crate::widget::unit::WidgetUnitNode::from($expr))
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
                |[ $listed_slot_widgets:expr ]|
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
                props = $crate::props::Props::from($props);
            )?
            #[allow(unused_mut)]
            let mut named_slots = std::collections::HashMap::new();
            $(
                $(
                    let widget = $crate::widget_wrap!{$named_slot_widget};
                    if widget.is_some() {
                        let name = stringify!($named_slot_name).to_owned();
                        named_slots.insert(name, widget);
                    }
                )+
            )?
            #[allow(unused_mut)]
            let mut listed_slots = vec![];
            $(
                listed_slots.extend($listed_slot_widgets);
            )?
            $(
                $(
                    let widget = $crate::widget_wrap!{$listed_slot_widget};
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
        $crate::widget!($tree)
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
    ($value:expr => $prop:ident) => {
        #[allow(unused_mut)]
        let mut $prop = match $value {
            $crate::widget::context::WidgetContext { $prop, .. } => $prop,
        };
    };
    ($value:expr => { $($prop:ident),+ }) => {
        #[allow(unused_mut)]
        let ( $( mut $prop ),+ ) = match $value {
            $crate::widget::context::WidgetContext { $( $prop ),+ , .. } => ( $( $prop ),+ ),
        };
    };
}

#[macro_export]
macro_rules! unpack_named_slots {
    ($map:expr => $name:ident) => {
        #[allow(unused_mut)]
        let mut $name = {
            let mut map = $map;
            match map.remove(stringify!($name)) {
                Some(widget) => widget,
                None => $crate::widget::node::WidgetNode::None,
            }
        };
    };
    ($map:expr => { $($name:ident),+ }) => {
        #[allow(unused_mut)]
        let ( $( mut $name ),+ ) = {
            let mut map = $map;
            (
                $(
                    {
                        match map.remove(stringify!($name)) {
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
    {
        $vis:vis $name:ident
        $( ( $( $param:ident ),+ ) )?
        $([ $( $hook:path ),+ $(,)? ])?
        $code:block
    } => {
        #[allow(unused_variables)]
        #[allow(unused_mut)]
        $vis fn $name(
            mut context: $crate::widget::context::WidgetContext
        ) -> $crate::widget::node::WidgetNode {
            {
                $(
                    $(
                        context.use_hook($hook);
                    ),+
                )?
            }
            {
                $(
                    $crate::unpack_context!(context => { $( $param ),+ });
                )?
                $code
            }
        }
    };
}

#[macro_export]
macro_rules! widget_hook {
    {
        $vis:vis $name:ident
        $( $param:ident )?
        $([ $( $hook:path ),+ $(,)? ])?
        $code:block
    } => {
        #[allow(unused_variables)]
        #[allow(unused_mut)]
        $vis fn $name(
            context: &mut $crate::widget::context::WidgetContext
        ) {
            {
                $(
                    $(
                        context.use_hook($hook);
                    ),+
                )?
            }
            {
                $(
                    $crate::unpack_context!(context => $param);
                )?
                $code
            }
        }
    };
    {
        $vis:vis $name:ident
        $( ( $( $param:ident ),+ ) )?
        $([ $( $hook:path ),+ $(,)? ])?
        $code:block
    } => {
        #[allow(unused_variables)]
        #[allow(unused_mut)]
        $vis fn $name(
            context: &mut $crate::widget::context::WidgetContext
        ) {
            {
                $(
                    $(
                        context.use_hook($hook);
                    ),+
                )?
            }
            {
                $(
                    $crate::unpack_context!(context => { $( $param ),+ });
                )?
                $code
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_id() {
        let id = WidgetId::new(
            "type".to_owned(),
            vec!["parent".to_owned(), "me".to_owned()],
        );
        assert_eq!(id.to_string(), "type:/parent/me".to_owned());
        assert_eq!(id.type_name(), "type");
        assert_eq!(id.parts().next().unwrap(), "parent");
        assert_eq!(id.key(), "me");
        assert_eq!(id.clone(), id);
    }
}
