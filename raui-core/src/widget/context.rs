use crate::{
    animator::{Animator, AnimatorStates},
    messenger::{MessageSender, Messenger},
    props::Props,
    signals::SignalSender,
    state::State,
    widget::{node::WidgetNode, WidgetId, WidgetLifeCycle},
};
use std::collections::HashMap;

pub struct WidgetContext<'a> {
    pub id: &'a WidgetId,
    pub key: &'a str,
    pub props: &'a Props,
    pub shared_props: &'a Props,
    pub state: State<'a>,
    pub animator: &'a AnimatorStates,
    pub life_cycle: &'a mut WidgetLifeCycle,
    pub named_slots: HashMap<String, WidgetNode>,
    pub listed_slots: Vec<WidgetNode>,
}

impl<'a> WidgetContext<'a> {
    pub fn take_named_slots(&mut self) -> HashMap<String, WidgetNode> {
        std::mem::replace(&mut self.named_slots, HashMap::new())
    }

    pub fn take_listed_slots(&mut self) -> Vec<WidgetNode> {
        std::mem::replace(&mut self.listed_slots, vec![])
    }

    pub fn use_hook<F>(&mut self, mut f: F) -> &mut Self
    where
        F: FnMut(&mut Self),
    {
        (f)(self);
        self
    }
}

impl<'a> std::fmt::Debug for WidgetContext<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WidgetContext")
            .field("id", &self.id)
            .field("key", &self.key)
            .field("props", &self.props)
            .field("shared_props", &self.shared_props)
            .field("named_slots", &self.named_slots)
            .field("listed_slots", &self.listed_slots)
            .finish()
    }
}

pub struct WidgetMountOrChangeContext<'a> {
    pub id: &'a WidgetId,
    pub props: &'a Props,
    pub shared_props: &'a Props,
    pub state: State<'a>,
    pub messenger: Messenger<'a>,
    pub signals: SignalSender,
    pub animator: Animator<'a>,
}

pub struct WidgetUnmountContext<'a> {
    pub id: &'a WidgetId,
    pub state: &'a Props,
    pub messenger: &'a MessageSender,
    pub signals: SignalSender,
}
