pub mod containers;
pub mod image_box;
pub mod interactive;
pub mod space_box;
pub mod text_box;

use crate::{
    MessageData, PrefabValue, PropsData, Scalar,
    messenger::Message,
    props::{Props, PropsData},
    widget::{
        FnWidget, WidgetId, WidgetIdOrRef, WidgetRef,
        context::WidgetContext,
        node::{WidgetNode, WidgetNodePrefab},
        utils::{Rect, Vec2},
    },
};
use intuicio_data::type_hash::TypeHash;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::TryFrom};

fn is_false(v: &bool) -> bool {
    !*v
}

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct MessageForwardProps {
    #[serde(default)]
    #[serde(skip_serializing_if = "WidgetIdOrRef::is_none")]
    pub to: WidgetIdOrRef,
    #[serde(default)]
    #[serde(skip)]
    pub types: Vec<TypeHash>,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub no_wrap: bool,
}

impl MessageForwardProps {
    pub fn with_type<T>(mut self) -> Self {
        self.types.push(TypeHash::of::<T>());
        self
    }
}

#[derive(MessageData, Debug, Clone)]
#[message_data(crate::messenger::MessageData)]
pub struct ForwardedMessage {
    pub sender: WidgetId,
    pub data: Message,
}

pub fn use_message_forward(context: &mut WidgetContext) {
    context.life_cycle.change(|context| {
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
            let t = msg.type_hash();
            if types.contains(&t) {
                if no_wrap {
                    context
                        .messenger
                        .write_raw(id.to_owned(), msg.clone_message());
                } else {
                    context.messenger.write(
                        id.to_owned(),
                        ForwardedMessage {
                            sender: context.id.to_owned(),
                            data: msg.clone_message(),
                        },
                    );
                }
            }
        }
    });
}

#[derive(MessageData, Debug, Copy, Clone, PartialEq)]
#[message_data(crate::messenger::MessageData)]
pub enum ResizeListenerSignal {
    Register,
    Unregister,
    Change(Vec2),
}

pub fn use_resize_listener(context: &mut WidgetContext) {
    context.life_cycle.mount(|context| {
        context.signals.write(ResizeListenerSignal::Register);
    });

    context.life_cycle.unmount(|context| {
        context.signals.write(ResizeListenerSignal::Unregister);
    });
}

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct RelativeLayoutProps {
    #[serde(default)]
    #[serde(skip_serializing_if = "WidgetIdOrRef::is_none")]
    pub relative_to: WidgetIdOrRef,
}

#[derive(MessageData, Debug, Clone, PartialEq)]
#[message_data(crate::messenger::MessageData)]
pub enum RelativeLayoutListenerSignal {
    /// (relative to id)
    Register(WidgetId),
    Unregister,
    /// (outer box size, inner box rect)
    Change(Vec2, Rect),
}

pub fn use_relative_layout_listener(context: &mut WidgetContext) {
    context.life_cycle.mount(|context| {
        if let Ok(props) = context.props.read::<RelativeLayoutProps>() {
            if let Some(relative_to) = props.relative_to.read() {
                context
                    .signals
                    .write(RelativeLayoutListenerSignal::Register(relative_to));
            }
        }
    });

    // TODO: when user will change widget IDs after mounting, we might want to re-register
    // this widget with new IDs.

    context.life_cycle.unmount(|context| {
        context
            .signals
            .write(RelativeLayoutListenerSignal::Unregister);
    });
}

#[derive(PropsData, Debug, Copy, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct WidgetAlpha(pub Scalar);

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
    pub fn new(processor: FnWidget, type_name: impl ToString) -> Self {
        Self {
            processor,
            type_name: type_name.to_string(),
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

    pub fn maybe_idref<T>(mut self, v: Option<T>) -> Self
    where
        T: Into<WidgetRef>,
    {
        self.idref = v.map(|v| v.into());
        self
    }

    pub fn with_props<T>(mut self, v: T) -> Self
    where
        T: 'static + PropsData,
    {
        self.props.write(v);
        self
    }

    pub fn maybe_with_props<T>(self, v: Option<T>) -> Self
    where
        T: 'static + PropsData,
    {
        if let Some(v) = v {
            self.with_props(v)
        } else {
            self
        }
    }

    pub fn merge_props(mut self, v: Props) -> Self {
        let props = std::mem::take(&mut self.props);
        self.props = props.merge(v);
        self
    }

    pub fn with_shared_props<T>(mut self, v: T) -> Self
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

    pub fn maybe_with_shared_props<T>(self, v: Option<T>) -> Self
    where
        T: 'static + PropsData,
    {
        if let Some(v) = v {
            self.with_shared_props(v)
        } else {
            self
        }
    }

    pub fn merge_shared_props(mut self, v: Props) -> Self {
        if let Some(props) = self.shared_props.take() {
            self.shared_props = Some(props.merge(v));
        } else {
            self.shared_props = Some(v);
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

    pub fn maybe_listed_slot<T>(mut self, v: Option<T>) -> Self
    where
        T: Into<WidgetNode>,
    {
        if let Some(v) = v {
            self.listed_slots.push(v.into());
        }
        self
    }

    pub fn listed_slots<I, T>(mut self, v: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<WidgetNode>,
    {
        self.listed_slots.extend(v.into_iter().map(|v| v.into()));
        self
    }

    pub fn named_slot<T>(mut self, k: impl ToString, v: T) -> Self
    where
        T: Into<WidgetNode>,
    {
        self.named_slots.insert(k.to_string(), v.into());
        self
    }

    pub fn maybe_named_slot<T>(mut self, k: impl ToString, v: Option<T>) -> Self
    where
        T: Into<WidgetNode>,
    {
        if let Some(v) = v {
            self.named_slots.insert(k.to_string(), v.into());
        }
        self
    }

    pub fn named_slots<I, K, T>(mut self, v: I) -> Self
    where
        I: IntoIterator<Item = (K, T)>,
        K: ToString,
        T: Into<WidgetNode>,
    {
        self.named_slots
            .extend(v.into_iter().map(|(k, v)| (k.to_string(), v.into())));
        self
    }

    pub fn remap_props<F>(&mut self, mut f: F)
    where
        F: FnMut(Props) -> Props,
    {
        let props = std::mem::take(&mut self.props);
        self.props = (f)(props);
    }

    pub fn remap_shared_props<F>(&mut self, mut f: F)
    where
        F: FnMut(Props) -> Props,
    {
        if let Some(shared_props) = &mut self.shared_props {
            let props = std::mem::take(shared_props);
            *shared_props = (f)(props);
        } else {
            self.shared_props = Some((f)(Default::default()));
        }
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
