use crate::{
    layout::{Layout, LayoutEngine},
    messenger::{MessageReceiver, MessageSender, Messages, Messenger},
    renderer::Renderer,
    signals::{Signal, SignalReceiver, SignalSender},
    state::{State, StateData, StateUpdate},
    widget::{
        component::WidgetComponent, context::WidgetContext, node::WidgetNode, unit::WidgetUnit,
        utils::Rect, WidgetId, WidgetLifeCycle, WidgetUnmountClosure,
    },
};
use std::{
    collections::{HashMap, HashSet},
    convert::TryInto,
    sync::mpsc::{channel, Receiver, Sender},
};

pub struct Application {
    tree: WidgetNode,
    rendered_tree: WidgetUnit,
    layout: Layout,
    states: HashMap<WidgetId, StateData>,
    state_receivers: HashMap<WidgetId, Receiver<StateData>>,
    message_sender: MessageSender,
    message_receiver: MessageReceiver,
    signal_sender: Sender<Signal>,
    signal_receiver: SignalReceiver,
    unmount_closures: HashMap<WidgetId, Vec<Box<WidgetUnmountClosure>>>,
    dirty: bool,
    render_changed: bool,
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

impl Application {
    #[inline]
    pub fn new() -> Self {
        let (message_sender, message_receiver) = channel();
        let message_sender = MessageSender::new(message_sender);
        let message_receiver = MessageReceiver::new(message_receiver);
        let (signal_sender, signal_receiver) = channel();
        let signal_receiver = SignalReceiver::new(signal_receiver);
        Self {
            tree: Default::default(),
            rendered_tree: Default::default(),
            layout: Default::default(),
            states: Default::default(),
            state_receivers: Default::default(),
            message_sender,
            message_receiver,
            signal_sender,
            signal_receiver,
            unmount_closures: Default::default(),
            dirty: true,
            render_changed: false,
        }
    }

    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    #[inline]
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    #[inline]
    pub fn does_render_changed(&self) -> bool {
        self.render_changed
    }

    #[inline]
    pub fn tree(&self) -> &WidgetNode {
        &self.tree
    }

    #[inline]
    pub fn rendered_tree(&self) -> &WidgetUnit {
        &self.rendered_tree
    }

    #[inline]
    pub fn layout_data(&self) -> &Layout {
        &self.layout
    }

    #[inline]
    pub fn apply(&mut self, tree: WidgetNode) {
        self.tree = tree;
        self.dirty = true;
        self.process();
    }

    #[inline]
    pub fn render<R, T, E>(&self, renderer: &mut R) -> Result<T, E>
    where
        R: Renderer<T, E>,
    {
        renderer.render(&self.rendered_tree, &self.layout)
    }

    #[inline]
    pub fn render_change<R, T, E>(&mut self, renderer: &mut R) -> Result<Option<T>, E>
    where
        R: Renderer<T, E>,
    {
        if self.render_changed {
            Ok(Some(self.render(renderer)?))
        } else {
            Ok(None)
        }
    }

    #[inline]
    pub fn layout<L, E>(&mut self, ui_space: Rect, layout_engine: &mut L) -> Result<(), E>
    where
        L: LayoutEngine<E>,
    {
        self.layout = layout_engine.layout(ui_space, &self.rendered_tree)?;
        Ok(())
    }

    #[inline]
    pub fn layout_change<L, E>(&mut self, ui_space: Rect, layout_engine: &mut L) -> Result<bool, E>
    where
        L: LayoutEngine<E>,
    {
        if self.render_changed {
            self.layout(ui_space, layout_engine)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    #[inline]
    pub fn messenger(&self) -> &MessageSender {
        &self.message_sender
    }

    #[inline]
    pub fn signals(&self) -> &SignalReceiver {
        &self.signal_receiver
    }

    #[inline]
    pub fn forced_process(&mut self) -> bool {
        self.dirty = true;
        self.process()
    }

    pub fn process(&mut self) -> bool {
        self.render_changed = false;
        let changed_states = self
            .state_receivers
            .iter()
            .filter_map(|(id, receiver)| {
                receiver.try_iter().last().map(|state| (id.clone(), state))
            })
            .collect::<HashMap<_, _>>();
        let mut messages = self.message_receiver.process();
        if !self.dirty && changed_states.is_empty() && messages.is_empty() {
            return false;
        }
        self.dirty = false;
        self.state_receivers.clear();
        let old_states = std::mem::replace(&mut self.states, HashMap::new());
        let states = old_states
            .into_iter()
            .chain(changed_states.into_iter())
            .collect::<HashMap<_, _>>();
        let tree = self.tree.clone();
        let mut used_ids = HashSet::new();
        let mut new_states = HashMap::new();
        let rendered_tree = self.process_node(
            tree,
            &states,
            vec![],
            &mut messages,
            &mut new_states,
            &mut used_ids,
            "<*>".to_string(),
        );
        self.states = states
            .into_iter()
            .chain(new_states.into_iter())
            .filter(|(id, state)| {
                if used_ids.contains(id) {
                    true
                } else {
                    if let Some(closures) = self.unmount_closures.remove(id) {
                        for mut closure in closures {
                            let message_sender = &self.message_sender;
                            let signal_sender =
                                SignalSender::new(id.clone(), self.signal_sender.clone());
                            (closure)(id, state, message_sender, &signal_sender);
                        }
                    }
                    false
                }
            })
            .collect();
        if let Ok(tree) = rendered_tree.try_into() {
            self.rendered_tree = tree;
            true
        } else {
            false
        }
    }

    fn process_node<'a>(
        &mut self,
        node: WidgetNode,
        states: &'a HashMap<WidgetId, StateData>,
        mut path: Vec<String>,
        messages: &mut HashMap<WidgetId, Messages>,
        new_states: &mut HashMap<WidgetId, StateData>,
        used_ids: &mut HashSet<WidgetId>,
        possible_key: String,
    ) -> WidgetNode {
        match node {
            WidgetNode::Component(component) => {
                let WidgetComponent {
                    processor,
                    type_name,
                    key,
                    props,
                    listed_slots,
                    named_slots,
                } = component;
                let key = match &key {
                    Some(key) => key.to_owned(),
                    None => possible_key.to_owned(),
                };
                path.push(key.clone());
                let listed_slots = listed_slots
                    .into_iter()
                    .enumerate()
                    .map(|(i, node)| {
                        self.process_node(
                            node,
                            states,
                            path.clone(),
                            messages,
                            new_states,
                            used_ids,
                            format!("<{}>", i),
                        )
                    })
                    .filter(|node| node.is_some())
                    .collect::<Vec<_>>();
                let named_slots = named_slots
                    .into_iter()
                    .enumerate()
                    .map(|(i, (name, node))| {
                        (
                            name.to_owned(),
                            self.process_node(
                                node,
                                states,
                                path.clone(),
                                messages,
                                new_states,
                                used_ids,
                                format!("<{}:{}>", i, name),
                            ),
                        )
                    })
                    .filter(|(_, node)| node.is_some())
                    .collect::<HashMap<_, _>>();
                let id = WidgetId::new(type_name.to_owned(), path.clone());
                used_ids.insert(id.clone());
                let (sender, receiver) = channel();
                let messages_list = match messages.remove(&id) {
                    Some(messages) => messages,
                    None => Messages::new(),
                };
                let mut life_cycle = WidgetLifeCycle::default();
                let (new_node, mounted) = match states.get(&id) {
                    Some(state) => {
                        let state = State::new(state, StateUpdate::new(sender.clone()));
                        let context = WidgetContext {
                            id: &id,
                            key: &key,
                            props: &props,
                            state,
                            life_cycle: &mut life_cycle,
                            named_slots,
                            listed_slots,
                        };
                        ((processor)(context), false)
                    }
                    None => {
                        let state_data = Box::new(()) as StateData;
                        let state = State::new(&state_data, StateUpdate::new(sender.clone()));
                        let context = WidgetContext {
                            id: &id,
                            key: &key,
                            props: &props,
                            state,
                            life_cycle: &mut life_cycle,
                            named_slots,
                            listed_slots,
                        };
                        let node = (processor)(context);
                        new_states.insert(id.clone(), state_data);
                        (node, true)
                    }
                };
                let (mount, change, unmount) = life_cycle.unwrap();
                if mounted {
                    if !mount.is_empty() {
                        if let Some(state) = new_states.get(&id) {
                            let state = State::new(state, StateUpdate::new(sender.clone()));
                            let messenger =
                                Messenger::new(self.message_sender.clone(), &messages_list);
                            let signal_sender =
                                SignalSender::new(id.clone(), self.signal_sender.clone());
                            for mut closure in mount {
                                (closure)(&id, &props, &state, &messenger, &signal_sender);
                            }
                        }
                    }
                } else if !change.is_empty() {
                    if let Some(state) = states.get(&id) {
                        let state = State::new(state, StateUpdate::new(sender.clone()));
                        let messenger = Messenger::new(self.message_sender.clone(), &messages_list);
                        let signal_sender =
                            SignalSender::new(id.clone(), self.signal_sender.clone());
                        for mut closure in change {
                            (closure)(&id, &props, &state, &messenger, &signal_sender);
                        }
                    }
                }
                if !unmount.is_empty() {
                    self.unmount_closures.insert(id.clone(), unmount);
                }
                let new_node = self.process_node(
                    new_node,
                    states,
                    path,
                    messages,
                    new_states,
                    used_ids,
                    possible_key,
                );
                self.state_receivers.insert(id, receiver);
                new_node
            }
            _ => node,
        }
    }
}
