use crate::{
    messenger::Messenger,
    props::Props,
    signals::SignalSender,
    state::State,
    widget::{node::WidgetNode, WidgetId, WidgetLifeCycle},
};
use std::collections::HashMap;

pub struct WidgetContext<'a> {
    pub id: WidgetId,
    pub key: &'a str,
    pub props: &'a Props,
    pub state: State<'a>,
    pub messenger: Messenger,
    pub signals: SignalSender,
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
            .field("named_slots", &self.named_slots)
            .field("listed_slots", &self.listed_slots)
            .finish()
    }
}
