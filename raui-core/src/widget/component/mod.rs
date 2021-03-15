pub mod containers;
pub mod image_box;
pub mod interactive;
pub mod space_box;
pub mod text_box;

use crate::{
    messenger::Message,
    props::{Props, PropsData},
    widget::{
        node::{WidgetNode, WidgetNodePrefab},
        FnWidget, WidgetId, WidgetIdOrRef, WidgetRef,
    },
    widget_hook, PrefabValue, Scalar,
};
use serde::{Deserialize, Serialize};
use std::{any::TypeId, collections::HashMap, convert::TryFrom};

fn is_false(v: &bool) -> bool {
    !*v
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MessageForwardProps {
    #[serde(default)]
    #[serde(skip_serializing_if = "WidgetIdOrRef::is_none")]
    pub to: WidgetIdOrRef,
    #[serde(default)]
    #[serde(skip)]
    pub types: Vec<TypeId>,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub no_wrap: bool,
}
implement_props_data!(MessageForwardProps);

#[derive(Debug, Clone)]
pub struct ForwardedMessage {
    pub sender: WidgetId,
    pub data: Message,
}
implement_message_data!(ForwardedMessage);

widget_hook! {
    pub use_message_forward(life_cycle) {
        life_cycle.change(|context| {
            let (id, no_wrap, types) = match context.props.read::<MessageForwardProps>() {
                Ok(forward) => match forward.to.read() {
                    Some(id) => (id, forward.no_wrap, &forward.types),
                    _ => return,
                },
                _ => match context.shared_props.read::<MessageForwardProps>() {
                    Ok(forward) => match forward.to.read() {
                        Some(id) => (id, forward.no_wrap, &forward.types),
                        _ => return,
                    },
                    _ => return,
                },
            };
            for msg in context.messenger.messages {
                let t = msg.as_any().type_id();
                if types.contains(&t) {
                    if no_wrap {
                        context.messenger.write_raw(id.to_owned(), msg.clone_message());
                    } else {
                        context.messenger.write(id.to_owned(), ForwardedMessage {
                            sender: context.id.to_owned(),
                            data: msg.clone_message(),
                        });
                    }
                }
            }
        });
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ResizeListenerSignal {
    Register,
    Unregister,
    Change,
}
implement_message_data!(ResizeListenerSignal);

widget_hook! {
    pub use_resize_listener(life_cycle) {
        life_cycle.mount(|context| {
            context.signals.write(ResizeListenerSignal::Register);
        });

        life_cycle.unmount(|context| {
            context.signals.write(ResizeListenerSignal::Unregister);
        });
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct WidgetAlpha(pub Scalar);
implement_props_data!(WidgetAlpha);

impl Default for WidgetAlpha {
    fn default() -> Self {
        Self(1.0)
    }
}

impl WidgetAlpha {
    pub fn multiply(&mut self, alpha: Scalar) {
        self.0 *= alpha;
    }
}

#[derive(Clone)]
pub struct WidgetComponent {
    pub processor: FnWidget,
    pub type_name: String,
    pub key: Option<String>,
    pub idref: Option<WidgetRef>,
    pub props: Props,
    pub shared_props: Option<Props>,
    pub listed_slots: Vec<WidgetNode>,
    pub named_slots: HashMap<String, WidgetNode>,
}

impl WidgetComponent {
    pub fn new(processor: FnWidget, type_name: &str) -> Self {
        Self {
            processor,
            type_name: type_name.to_owned(),
            key: None,
            idref: None,
            props: Props::default(),
            shared_props: None,
            listed_slots: Vec::new(),
            named_slots: HashMap::new(),
        }
    }

    pub fn key<T>(mut self, v: T) -> Self
    where
        T: ToString,
    {
        self.key = Some(v.to_string());
        self
    }

    pub fn idref<T>(mut self, v: T) -> Self
    where
        T: Into<WidgetRef>,
    {
        self.idref = Some(v.into());
        self
    }

    pub fn props<T>(mut self, v: T) -> Self
    where
        T: 'static + PropsData,
    {
        self.props.write(v);
        self
    }

    pub fn shared_props<T>(mut self, v: T) -> Self
    where
        T: 'static + PropsData,
    {
        if let Some(props) = &mut self.shared_props {
            props.write(v);
        } else {
            self.shared_props = Some(Props::new(v));
        }
        self
    }

    pub fn listed_slot<T>(mut self, v: T) -> Self
    where
        T: Into<WidgetNode>,
    {
        self.listed_slots.push(v.into());
        self
    }

    pub fn named_slot<T>(mut self, k: &str, v: T) -> Self
    where
        T: Into<WidgetNode>,
    {
        self.named_slots.insert(k.to_owned(), v.into());
        self
    }

    pub fn remap_props<F>(&mut self, mut f: F)
    where
        F: FnMut(Props) -> Props,
    {
        let props = std::mem::take(&mut self.props);
        self.props = (f)(props);
    }
}

impl std::fmt::Debug for WidgetComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("WidgetComponent");
        s.field("type_name", &self.type_name);
        if let Some(key) = &self.key {
            s.field("key", key);
        }
        s.field("props", &self.props);
        s.field("shared_props", &self.shared_props);
        if !self.listed_slots.is_empty() {
            s.field("listed_slots", &self.listed_slots);
        }
        if !self.named_slots.is_empty() {
            s.field("named_slots", &self.named_slots);
        }
        s.finish()
    }
}

impl TryFrom<WidgetNode> for WidgetComponent {
    type Error = ();

    fn try_from(node: WidgetNode) -> Result<Self, Self::Error> {
        if let WidgetNode::Component(v) = node {
            Ok(v)
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub(crate) struct WidgetComponentPrefab {
    #[serde(default)]
    pub type_name: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(default)]
    pub props: PrefabValue,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared_props: Option<PrefabValue>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub listed_slots: Vec<WidgetNodePrefab>,
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub named_slots: HashMap<String, WidgetNodePrefab>,
}
