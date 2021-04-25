use crate::{
    animator::{AnimationUpdate, Animator, AnimatorStates},
    interactive::InteractionsEngine,
    layout::{CoordsMapping, Layout, LayoutEngine},
    messenger::{Message, MessageData, MessageSender, Messages, Messenger},
    props::{Props, PropsData, PropsRegistry},
    renderer::Renderer,
    signals::{Signal, SignalSender},
    state::{State, StateUpdate},
    widget::{
        component::{WidgetComponent, WidgetComponentPrefab},
        context::{WidgetContext, WidgetMountOrChangeContext, WidgetUnmountContext},
        node::{WidgetNode, WidgetNodePrefab},
        unit::{
            area::{AreaBoxNode, AreaBoxNodePrefab},
            content::{
                ContentBoxItem, ContentBoxItemNode, ContentBoxItemNodePrefab, ContentBoxNode,
                ContentBoxNodePrefab,
            },
            flex::{
                FlexBoxItem, FlexBoxItemNode, FlexBoxItemNodePrefab, FlexBoxNode, FlexBoxNodePrefab,
            },
            grid::{
                GridBoxItem, GridBoxItemNode, GridBoxItemNodePrefab, GridBoxNode, GridBoxNodePrefab,
            },
            image::{ImageBoxNode, ImageBoxNodePrefab},
            portal::{
                PortalBox, PortalBoxNode, PortalBoxNodePrefab, PortalBoxSlot, PortalBoxSlotNode,
                PortalBoxSlotNodePrefab,
            },
            size::{SizeBoxNode, SizeBoxNodePrefab},
            text::{TextBoxNode, TextBoxNodePrefab},
            WidgetUnit, WidgetUnitNode, WidgetUnitNodePrefab,
        },
        FnWidget, WidgetId, WidgetLifeCycle,
    },
    Prefab, PrefabError, PrefabValue, Scalar,
};
use std::{
    collections::{HashMap, HashSet},
    convert::TryInto,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Sender},
        Arc,
    },
};

#[derive(Debug, Default, Clone)]
pub struct ChangeNotifier(Arc<AtomicBool>);

impl ChangeNotifier {
    pub fn new(changed: bool) -> Self {
        Self(Arc::new(AtomicBool::new(changed)))
    }

    pub fn change(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }

    pub fn has_changed(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }

    pub fn consume_change(&mut self) -> bool {
        self.0.swap(false, Ordering::Relaxed)
    }
}

#[derive(Debug, Clone)]
pub enum ApplicationError {
    Prefab(PrefabError),
    ComponentMappingNotFound(String),
}

impl From<PrefabError> for ApplicationError {
    fn from(error: PrefabError) -> Self {
        Self::Prefab(error)
    }
}

#[derive(Debug, Clone)]
pub enum InvalidationCause {
    None,
    Forced,
    StateChange(WidgetId),
    MessageReceived(WidgetId),
    AnimationInProgress(WidgetId),
}

impl Default for InvalidationCause {
    fn default() -> Self {
        Self::None
    }
}

pub struct Application {
    component_mappings: HashMap<String, FnWidget>,
    props_registry: PropsRegistry,
    tree: WidgetNode,
    rendered_tree: WidgetUnit,
    layout: Layout,
    states: HashMap<WidgetId, Props>,
    state_changes: HashMap<WidgetId, Props>,
    animators: HashMap<WidgetId, AnimatorStates>,
    messages: HashMap<WidgetId, Messages>,
    signals: Vec<Signal>,
    #[allow(clippy::type_complexity)]
    unmount_closures: HashMap<WidgetId, Vec<Box<dyn FnMut(WidgetUnmountContext) + Send + Sync>>>,
    dirty: bool,
    render_changed: bool,
    last_invalidation_cause: InvalidationCause,
    change_notifier: ChangeNotifier,
    pub animations_delta_time: Scalar,
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

impl Application {
    #[inline]
    pub fn new() -> Self {
        Self {
            component_mappings: Default::default(),
            props_registry: Default::default(),
            tree: Default::default(),
            rendered_tree: Default::default(),
            layout: Default::default(),
            states: Default::default(),
            state_changes: Default::default(),
            animators: Default::default(),
            messages: Default::default(),
            signals: Default::default(),
            unmount_closures: Default::default(),
            dirty: true,
            render_changed: false,
            last_invalidation_cause: Default::default(),
            change_notifier: ChangeNotifier::default(),
            animations_delta_time: 0.0,
        }
    }

    #[inline]
    pub fn setup<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Self),
    {
        (f)(self);
    }

    #[inline]
    pub fn change_notifier(&self) -> ChangeNotifier {
        self.change_notifier.clone()
    }

    #[inline]
    pub fn register_component(&mut self, type_name: &str, processor: FnWidget) {
        self.component_mappings
            .insert(type_name.to_owned(), processor);
    }

    #[inline]
    pub fn unregister_component(&mut self, type_name: &str) {
        self.component_mappings.remove(type_name);
    }

    #[inline]
    pub fn register_props<T>(&mut self, name: &str)
    where
        T: 'static + Prefab + PropsData,
    {
        self.props_registry.register_factory::<T>(name);
    }

    #[inline]
    pub fn unregister_props(&mut self, name: &str) {
        self.props_registry.unregister_factory(name);
    }

    #[inline]
    pub fn serialize_props(&self, props: &Props) -> Result<PrefabValue, PrefabError> {
        self.props_registry.serialize(props)
    }

    #[inline]
    pub fn deserialize_props(&self, data: PrefabValue) -> Result<Props, PrefabError> {
        self.props_registry.deserialize(data)
    }

    #[inline]
    pub fn serialize_node(&self, data: &WidgetNode) -> Result<PrefabValue, ApplicationError> {
        Ok(self.node_to_prefab(data)?.to_prefab()?)
    }

    #[inline]
    pub fn deserialize_node(&self, data: PrefabValue) -> Result<WidgetNode, ApplicationError> {
        self.node_from_prefab(WidgetNodePrefab::from_prefab(data)?)
    }

    #[inline]
    pub fn last_invalidation_cause(&self) -> &InvalidationCause {
        &self.last_invalidation_cause
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
    pub fn has_layout_widget(&self, id: &WidgetId) -> bool {
        self.layout.items.keys().any(|k| k == id)
    }

    #[inline]
    pub fn apply(&mut self, tree: WidgetNode) {
        self.tree = tree;
        self.dirty = true;
    }

    #[inline]
    pub fn render<R, T, E>(&self, mapping: &CoordsMapping, renderer: &mut R) -> Result<T, E>
    where
        R: Renderer<T, E>,
    {
        renderer.render(&self.rendered_tree, mapping, &self.layout)
    }

    #[inline]
    pub fn render_change<R, T, E>(
        &mut self,
        mapping: &CoordsMapping,
        renderer: &mut R,
    ) -> Result<Option<T>, E>
    where
        R: Renderer<T, E>,
    {
        if self.render_changed {
            Ok(Some(self.render(mapping, renderer)?))
        } else {
            Ok(None)
        }
    }

    #[inline]
    pub fn layout<L, E>(&mut self, mapping: &CoordsMapping, layout_engine: &mut L) -> Result<(), E>
    where
        L: LayoutEngine<E>,
    {
        self.layout = layout_engine.layout(mapping, &self.rendered_tree)?;
        Ok(())
    }

    #[inline]
    pub fn layout_change<L, E>(
        &mut self,
        mapping: &CoordsMapping,
        layout_engine: &mut L,
    ) -> Result<bool, E>
    where
        L: LayoutEngine<E>,
    {
        if self.render_changed {
            self.layout(mapping, layout_engine)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    #[inline]
    pub fn interact<I, R, E>(&mut self, interactions_engine: &mut I) -> Result<R, E>
    where
        I: InteractionsEngine<R, E>,
    {
        interactions_engine.perform_interactions(self)
    }

    #[inline]
    pub fn send_message<T>(&mut self, id: &WidgetId, data: T)
    where
        T: 'static + MessageData,
    {
        self.send_message_raw(id, Box::new(data));
    }

    #[inline]
    pub fn send_message_raw(&mut self, id: &WidgetId, data: Message) {
        if let Some(list) = self.messages.get_mut(id) {
            list.push(data);
        } else {
            self.messages.insert(id.to_owned(), vec![data]);
        }
    }

    #[inline]
    pub fn signals(&self) -> &[Signal] {
        &self.signals
    }

    #[inline]
    pub fn consume_signals(&mut self) -> Vec<Signal> {
        std::mem::take(&mut self.signals)
    }

    #[inline]
    pub fn state_read(&self, id: &WidgetId) -> Option<&Props> {
        self.states.get(id)
    }

    #[inline]
    pub fn state_write(&mut self, id: &WidgetId, data: Props) {
        if self.states.contains_key(id) {
            self.state_changes.insert(id.to_owned(), data);
        }
    }

    pub fn state_mutate<F>(&mut self, id: &WidgetId, mut f: F)
    where
        F: FnMut(&Props) -> Props,
    {
        if let Some(state) = self.states.get(id) {
            self.state_changes.insert(id.to_owned(), f(state));
        }
    }

    pub fn state_mutate_cloned<F>(&mut self, id: &WidgetId, mut f: F)
    where
        F: FnMut(&mut Props),
    {
        if let Some(mut state) = self.states.get(id).cloned() {
            f(&mut state);
            self.state_changes.insert(id.to_owned(), state);
        }
    }

    #[inline]
    pub fn forced_process(&mut self) -> bool {
        self.dirty = true;
        self.process()
    }

    pub fn process(&mut self) -> bool {
        if self.change_notifier.consume_change() {
            self.dirty = true;
        }
        self.animations_delta_time = self.animations_delta_time.max(0.0);
        self.last_invalidation_cause = InvalidationCause::None;
        self.render_changed = false;
        let changed_states = std::mem::take(&mut self.state_changes);
        let mut messages = std::mem::take(&mut self.messages);
        let changed_animators = self.animators.values().any(|a| a.in_progress());
        if !self.dirty && changed_states.is_empty() && messages.is_empty() && !changed_animators {
            return false;
        }
        if self.dirty {
            self.last_invalidation_cause = InvalidationCause::Forced;
        }
        if let Some((id, _)) = self.animators.iter().find(|(_, a)| a.in_progress()) {
            self.last_invalidation_cause = InvalidationCause::AnimationInProgress(id.to_owned());
        }
        if let Some((id, _)) = messages.iter().next() {
            self.last_invalidation_cause = InvalidationCause::MessageReceived(id.to_owned());
        }
        if let Some((id, _)) = changed_states.iter().next() {
            self.last_invalidation_cause = InvalidationCause::StateChange(id.to_owned());
        }
        let (message_sender, message_receiver) = channel();
        let message_sender = MessageSender::new(message_sender);
        for (k, a) in &mut self.animators {
            a.process(self.animations_delta_time, &k, &message_sender);
        }
        self.dirty = false;
        let old_states = std::mem::take(&mut self.states);
        let states = old_states
            .into_iter()
            .chain(changed_states.into_iter())
            .collect::<HashMap<_, _>>();
        let (signal_sender, signal_receiver) = channel();
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
            None,
            &message_sender,
            &signal_sender,
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
                            let messenger = &message_sender;
                            let signals = SignalSender::new(id.clone(), signal_sender.clone());
                            let context = WidgetUnmountContext {
                                id,
                                state,
                                messenger,
                                signals,
                            };
                            (closure)(context);
                        }
                    }
                    self.animators.remove(id);
                    false
                }
            })
            .collect();
        while let Ok((id, message)) = message_receiver.try_recv() {
            if let Some(list) = self.messages.get_mut(&id) {
                list.push(message);
            } else {
                self.messages.insert(id, vec![message]);
            }
        }
        self.signals.clear();
        while let Ok(data) = signal_receiver.try_recv() {
            self.signals.push(data);
        }
        self.animators = std::mem::take(&mut self.animators)
            .into_iter()
            .filter_map(|(k, a)| if a.in_progress() { Some((k, a)) } else { None })
            .collect::<HashMap<_, _>>();
        if let Ok(tree) = rendered_tree.try_into() {
            self.rendered_tree = Self::teleport_portals(tree);
            true
        } else {
            false
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn process_node<'a>(
        &mut self,
        node: WidgetNode,
        states: &'a HashMap<WidgetId, Props>,
        path: Vec<String>,
        messages: &mut HashMap<WidgetId, Messages>,
        new_states: &mut HashMap<WidgetId, Props>,
        used_ids: &mut HashSet<WidgetId>,
        possible_key: String,
        master_shared_props: Option<Props>,
        message_sender: &MessageSender,
        signal_sender: &Sender<Signal>,
    ) -> WidgetNode {
        match node {
            WidgetNode::None => node,
            WidgetNode::Component(component) => self.process_node_component(
                component,
                states,
                path,
                messages,
                new_states,
                used_ids,
                possible_key,
                master_shared_props,
                message_sender,
                signal_sender,
            ),
            WidgetNode::Unit(unit) => self.process_node_unit(
                unit,
                states,
                path,
                messages,
                new_states,
                used_ids,
                master_shared_props,
                message_sender,
                signal_sender,
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn process_node_component<'a>(
        &mut self,
        component: WidgetComponent,
        states: &'a HashMap<WidgetId, Props>,
        mut path: Vec<String>,
        messages: &mut HashMap<WidgetId, Messages>,
        new_states: &mut HashMap<WidgetId, Props>,
        used_ids: &mut HashSet<WidgetId>,
        possible_key: String,
        master_shared_props: Option<Props>,
        message_sender: &MessageSender,
        signal_sender: &Sender<Signal>,
    ) -> WidgetNode {
        let WidgetComponent {
            processor,
            type_name,
            key,
            mut idref,
            mut props,
            shared_props,
            listed_slots,
            named_slots,
        } = component;
        let mut shared_props = match (master_shared_props, shared_props) {
            (Some(master_shared_props), Some(shared_props)) => {
                master_shared_props.merge(shared_props)
            }
            (None, Some(shared_props)) => shared_props,
            (Some(master_shared_props), None) => master_shared_props,
            _ => Default::default(),
        };
        let key = match &key {
            Some(key) => key.to_owned(),
            None => possible_key.to_owned(),
        };
        path.push(key.clone());
        let id = WidgetId::new(&type_name, &path);
        used_ids.insert(id.clone());
        if let Some(idref) = &mut idref {
            idref.write(id.to_owned());
        }
        let (state_sender, state_receiver) = channel();
        let (animation_sender, animation_receiver) = channel();
        let messages_list = match messages.remove(&id) {
            Some(messages) => messages,
            None => Messages::new(),
        };
        let mut life_cycle = WidgetLifeCycle::default();
        let default_animator_state = AnimatorStates::default();
        let (new_node, mounted) = match states.get(&id) {
            Some(state) => {
                let state = State::new(state, StateUpdate::new(state_sender.clone()));
                let animator = self.animators.get(&id).unwrap_or(&default_animator_state);
                let context = WidgetContext {
                    id: &id,
                    idref: idref.as_ref(),
                    key: &key,
                    props: &mut props,
                    shared_props: &mut shared_props,
                    state,
                    animator,
                    life_cycle: &mut life_cycle,
                    named_slots,
                    listed_slots,
                };
                ((processor)(context), false)
            }
            None => {
                let state_data = Props::default();
                let state = State::new(&state_data, StateUpdate::new(state_sender.clone()));
                let animator = self.animators.get(&id).unwrap_or(&default_animator_state);
                let context = WidgetContext {
                    id: &id,
                    idref: idref.as_ref(),
                    key: &key,
                    props: &mut props,
                    shared_props: &mut shared_props,
                    state,
                    animator,
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
                    for mut closure in mount {
                        let state = State::new(state, StateUpdate::new(state_sender.clone()));
                        let messenger = Messenger::new(message_sender.clone(), &messages_list);
                        let signals = SignalSender::new(id.clone(), signal_sender.clone());
                        let animator = Animator::new(
                            self.animators.get(&id).unwrap_or(&default_animator_state),
                            AnimationUpdate::new(animation_sender.clone()),
                        );
                        let context = WidgetMountOrChangeContext {
                            id: &id,
                            props: &props,
                            shared_props: &shared_props,
                            state,
                            messenger,
                            signals,
                            animator,
                        };
                        (closure)(context);
                    }
                }
            }
        } else if !change.is_empty() {
            if let Some(state) = states.get(&id) {
                for mut closure in change {
                    let state = State::new(state, StateUpdate::new(state_sender.clone()));
                    let messenger = Messenger::new(message_sender.clone(), &messages_list);
                    let signals = SignalSender::new(id.clone(), signal_sender.clone());
                    let animator = Animator::new(
                        self.animators.get(&id).unwrap_or(&default_animator_state),
                        AnimationUpdate::new(animation_sender.clone()),
                    );
                    let context = WidgetMountOrChangeContext {
                        id: &id,
                        props: &props,
                        shared_props: &shared_props,
                        state,
                        messenger,
                        signals,
                        animator,
                    };
                    (closure)(context);
                }
            }
        }
        if !unmount.is_empty() {
            self.unmount_closures.insert(id.clone(), unmount);
        }
        while let Ok((name, data)) = animation_receiver.try_recv() {
            if let Some(states) = self.animators.get_mut(&id) {
                states.change(name, data);
            } else if let Some(data) = data {
                self.animators
                    .insert(id.to_owned(), AnimatorStates::new(name, data));
            }
        }
        let new_node = self.process_node(
            new_node,
            states,
            path,
            messages,
            new_states,
            used_ids,
            possible_key,
            Some(shared_props),
            message_sender,
            signal_sender,
        );
        while let Ok(data) = state_receiver.try_recv() {
            self.state_changes.insert(id.to_owned(), data);
        }
        new_node
    }

    #[allow(clippy::too_many_arguments)]
    fn process_node_unit<'a>(
        &mut self,
        mut unit: WidgetUnitNode,
        states: &'a HashMap<WidgetId, Props>,
        path: Vec<String>,
        messages: &mut HashMap<WidgetId, Messages>,
        new_states: &mut HashMap<WidgetId, Props>,
        used_ids: &mut HashSet<WidgetId>,
        master_shared_props: Option<Props>,
        message_sender: &MessageSender,
        signal_sender: &Sender<Signal>,
    ) -> WidgetNode {
        match &mut unit {
            WidgetUnitNode::None | WidgetUnitNode::ImageBox(_) | WidgetUnitNode::TextBox(_) => {}
            WidgetUnitNode::AreaBox(unit) => {
                let slot = *std::mem::take(&mut unit.slot);
                unit.slot = Box::new(self.process_node(
                    slot,
                    states,
                    path,
                    messages,
                    new_states,
                    used_ids,
                    ".".to_owned(),
                    master_shared_props,
                    message_sender,
                    signal_sender,
                ));
            }
            WidgetUnitNode::PortalBox(unit) => match &mut *unit.slot {
                PortalBoxSlotNode::Slot(data) => {
                    let slot = std::mem::take(data);
                    *data = self.process_node(
                        slot,
                        states,
                        path,
                        messages,
                        new_states,
                        used_ids,
                        ".".to_owned(),
                        master_shared_props,
                        message_sender,
                        signal_sender,
                    )
                }
                PortalBoxSlotNode::ContentItem(item) => {
                    let slot = std::mem::take(&mut item.slot);
                    item.slot = self.process_node(
                        slot,
                        states,
                        path,
                        messages,
                        new_states,
                        used_ids,
                        ".".to_owned(),
                        master_shared_props,
                        message_sender,
                        signal_sender,
                    )
                }
                PortalBoxSlotNode::FlexItem(item) => {
                    let slot = std::mem::take(&mut item.slot);
                    item.slot = self.process_node(
                        slot,
                        states,
                        path,
                        messages,
                        new_states,
                        used_ids,
                        ".".to_owned(),
                        master_shared_props,
                        message_sender,
                        signal_sender,
                    )
                }
                PortalBoxSlotNode::GridItem(item) => {
                    let slot = std::mem::take(&mut item.slot);
                    item.slot = self.process_node(
                        slot,
                        states,
                        path,
                        messages,
                        new_states,
                        used_ids,
                        ".".to_owned(),
                        master_shared_props,
                        message_sender,
                        signal_sender,
                    )
                }
            },
            WidgetUnitNode::ContentBox(unit) => {
                let items = std::mem::take(&mut unit.items);
                unit.items = items
                    .into_iter()
                    .enumerate()
                    .map(|(i, mut node)| {
                        let slot = std::mem::take(&mut node.slot);
                        node.slot = self.process_node(
                            slot,
                            states,
                            path.clone(),
                            messages,
                            new_states,
                            used_ids,
                            format!("<{}>", i),
                            master_shared_props.clone(),
                            message_sender,
                            signal_sender,
                        );
                        node
                    })
                    .collect::<Vec<_>>();
            }
            WidgetUnitNode::FlexBox(unit) => {
                let items = std::mem::take(&mut unit.items);
                unit.items = items
                    .into_iter()
                    .enumerate()
                    .map(|(i, mut node)| {
                        let slot = std::mem::take(&mut node.slot);
                        node.slot = self.process_node(
                            slot,
                            states,
                            path.clone(),
                            messages,
                            new_states,
                            used_ids,
                            format!("<{}>", i),
                            master_shared_props.clone(),
                            message_sender,
                            signal_sender,
                        );
                        node
                    })
                    .collect::<Vec<_>>();
            }
            WidgetUnitNode::GridBox(unit) => {
                let items = std::mem::take(&mut unit.items);
                unit.items = items
                    .into_iter()
                    .enumerate()
                    .map(|(i, mut node)| {
                        let slot = std::mem::take(&mut node.slot);
                        node.slot = self.process_node(
                            slot,
                            states,
                            path.clone(),
                            messages,
                            new_states,
                            used_ids,
                            format!("<{}>", i),
                            master_shared_props.clone(),
                            message_sender,
                            signal_sender,
                        );
                        node
                    })
                    .collect::<Vec<_>>();
            }
            WidgetUnitNode::SizeBox(unit) => {
                let slot = *std::mem::take(&mut unit.slot);
                unit.slot = Box::new(self.process_node(
                    slot,
                    states,
                    path,
                    messages,
                    new_states,
                    used_ids,
                    ".".to_owned(),
                    master_shared_props,
                    message_sender,
                    signal_sender,
                ));
            }
        }
        unit.into()
    }

    fn teleport_portals(mut root: WidgetUnit) -> WidgetUnit {
        let count = Self::estimate_portals(&root);
        if count == 0 {
            return root;
        }
        let mut portals = Vec::with_capacity(count);
        Self::consume_portals(&mut root, &mut portals);
        Self::inject_portals(&mut root, portals);
        root
    }

    fn estimate_portals(unit: &WidgetUnit) -> usize {
        let mut count = 0;
        match unit {
            WidgetUnit::None | WidgetUnit::ImageBox(_) | WidgetUnit::TextBox(_) => {}
            WidgetUnit::AreaBox(b) => count += Self::estimate_portals(&b.slot),
            WidgetUnit::PortalBox(b) => {
                count += Self::estimate_portals(match &*b.slot {
                    PortalBoxSlot::Slot(slot) => slot,
                    PortalBoxSlot::ContentItem(item) => &item.slot,
                    PortalBoxSlot::FlexItem(item) => &item.slot,
                    PortalBoxSlot::GridItem(item) => &item.slot,
                }) + 1
            }
            WidgetUnit::ContentBox(b) => {
                for item in &b.items {
                    count += Self::estimate_portals(&item.slot);
                }
            }
            WidgetUnit::FlexBox(b) => {
                for item in &b.items {
                    count += Self::estimate_portals(&item.slot);
                }
            }
            WidgetUnit::GridBox(b) => {
                for item in &b.items {
                    count += Self::estimate_portals(&item.slot);
                }
            }
            WidgetUnit::SizeBox(b) => count += Self::estimate_portals(&b.slot),
        }
        count
    }

    fn consume_portals(unit: &mut WidgetUnit, bucket: &mut Vec<(WidgetId, PortalBoxSlot)>) {
        match unit {
            WidgetUnit::None | WidgetUnit::ImageBox(_) | WidgetUnit::TextBox(_) => {}
            WidgetUnit::AreaBox(b) => Self::consume_portals(&mut b.slot, bucket),
            WidgetUnit::PortalBox(b) => {
                let PortalBox {
                    owner, mut slot, ..
                } = std::mem::take(b);
                Self::consume_portals(
                    match &mut *slot {
                        PortalBoxSlot::Slot(slot) => slot,
                        PortalBoxSlot::ContentItem(item) => &mut item.slot,
                        PortalBoxSlot::FlexItem(item) => &mut item.slot,
                        PortalBoxSlot::GridItem(item) => &mut item.slot,
                    },
                    bucket,
                );
                bucket.push((owner, *slot));
            }
            WidgetUnit::ContentBox(b) => {
                for item in &mut b.items {
                    Self::consume_portals(&mut item.slot, bucket);
                }
            }
            WidgetUnit::FlexBox(b) => {
                for item in &mut b.items {
                    Self::consume_portals(&mut item.slot, bucket);
                }
            }
            WidgetUnit::GridBox(b) => {
                for item in &mut b.items {
                    Self::consume_portals(&mut item.slot, bucket);
                }
            }
            WidgetUnit::SizeBox(b) => Self::consume_portals(&mut b.slot, bucket),
        }
    }

    fn inject_portals(unit: &mut WidgetUnit, mut portals: Vec<(WidgetId, PortalBoxSlot)>) {
        if portals.is_empty() {
            return;
        }
        if let Some(data) = unit.as_data() {
            if let Some(index) = portals.iter().position(|(id, _)| data.id() == id) {
                let slot = portals.swap_remove(index).1;
                match unit {
                    WidgetUnit::None
                    | WidgetUnit::PortalBox(_)
                    | WidgetUnit::ImageBox(_)
                    | WidgetUnit::TextBox(_) => {}
                    WidgetUnit::AreaBox(b) => match slot {
                        PortalBoxSlot::Slot(slot) => b.slot = Box::new(slot),
                        PortalBoxSlot::ContentItem(item) => b.slot = Box::new(item.slot),
                        PortalBoxSlot::FlexItem(item) => b.slot = Box::new(item.slot),
                        PortalBoxSlot::GridItem(item) => b.slot = Box::new(item.slot),
                    },
                    WidgetUnit::ContentBox(b) => b.items.push(match slot {
                        PortalBoxSlot::Slot(slot) => ContentBoxItem {
                            slot,
                            ..Default::default()
                        },
                        PortalBoxSlot::ContentItem(item) => item,
                        PortalBoxSlot::FlexItem(item) => ContentBoxItem {
                            slot: item.slot,
                            ..Default::default()
                        },
                        PortalBoxSlot::GridItem(item) => ContentBoxItem {
                            slot: item.slot,
                            ..Default::default()
                        },
                    }),
                    WidgetUnit::FlexBox(b) => b.items.push(match slot {
                        PortalBoxSlot::Slot(slot) => FlexBoxItem {
                            slot,
                            ..Default::default()
                        },
                        PortalBoxSlot::ContentItem(item) => FlexBoxItem {
                            slot: item.slot,
                            ..Default::default()
                        },
                        PortalBoxSlot::FlexItem(item) => item,
                        PortalBoxSlot::GridItem(item) => FlexBoxItem {
                            slot: item.slot,
                            ..Default::default()
                        },
                    }),
                    WidgetUnit::GridBox(b) => b.items.push(match slot {
                        PortalBoxSlot::Slot(slot) => GridBoxItem {
                            slot,
                            ..Default::default()
                        },
                        PortalBoxSlot::ContentItem(item) => GridBoxItem {
                            slot: item.slot,
                            ..Default::default()
                        },
                        PortalBoxSlot::FlexItem(item) => GridBoxItem {
                            slot: item.slot,
                            ..Default::default()
                        },
                        PortalBoxSlot::GridItem(item) => item,
                    }),
                    WidgetUnit::SizeBox(b) => match slot {
                        PortalBoxSlot::Slot(slot) => b.slot = Box::new(slot),
                        PortalBoxSlot::ContentItem(item) => b.slot = Box::new(item.slot),
                        PortalBoxSlot::FlexItem(item) => b.slot = Box::new(item.slot),
                        PortalBoxSlot::GridItem(item) => b.slot = Box::new(item.slot),
                    },
                }
            }
        }
    }

    fn node_to_prefab(&self, data: &WidgetNode) -> Result<WidgetNodePrefab, ApplicationError> {
        Ok(match data {
            WidgetNode::None => WidgetNodePrefab::None,
            WidgetNode::Component(data) => {
                WidgetNodePrefab::Component(self.component_to_prefab(data)?)
            }
            WidgetNode::Unit(data) => WidgetNodePrefab::Unit(self.unit_to_prefab(data)?),
        })
    }

    fn component_to_prefab(
        &self,
        data: &WidgetComponent,
    ) -> Result<WidgetComponentPrefab, ApplicationError> {
        if self.component_mappings.contains_key(&data.type_name) {
            Ok(WidgetComponentPrefab {
                type_name: data.type_name.to_owned(),
                key: data.key.clone(),
                props: self.props_registry.serialize(&data.props)?,
                shared_props: match &data.shared_props {
                    Some(p) => Some(self.props_registry.serialize(p)?),
                    None => None,
                },
                listed_slots: data
                    .listed_slots
                    .iter()
                    .map(|v| self.node_to_prefab(v))
                    .collect::<Result<_, _>>()?,
                named_slots: data
                    .named_slots
                    .iter()
                    .map(|(k, v)| Ok((k.to_owned(), self.node_to_prefab(v)?)))
                    .collect::<Result<_, ApplicationError>>()?,
            })
        } else {
            Err(ApplicationError::ComponentMappingNotFound(
                data.type_name.to_owned(),
            ))
        }
    }

    fn unit_to_prefab(
        &self,
        data: &WidgetUnitNode,
    ) -> Result<WidgetUnitNodePrefab, ApplicationError> {
        Ok(match data {
            WidgetUnitNode::None => WidgetUnitNodePrefab::None,
            WidgetUnitNode::AreaBox(data) => {
                WidgetUnitNodePrefab::AreaBox(self.area_box_to_prefab(data)?)
            }
            WidgetUnitNode::PortalBox(data) => {
                WidgetUnitNodePrefab::PortalBox(self.portal_box_to_prefab(data)?)
            }
            WidgetUnitNode::ContentBox(data) => {
                WidgetUnitNodePrefab::ContentBox(self.content_box_to_prefab(data)?)
            }
            WidgetUnitNode::FlexBox(data) => {
                WidgetUnitNodePrefab::FlexBox(self.flex_box_to_prefab(data)?)
            }
            WidgetUnitNode::GridBox(data) => {
                WidgetUnitNodePrefab::GridBox(self.grid_box_to_prefab(data)?)
            }
            WidgetUnitNode::SizeBox(data) => {
                WidgetUnitNodePrefab::SizeBox(self.size_box_to_prefab(data)?)
            }
            WidgetUnitNode::ImageBox(data) => {
                WidgetUnitNodePrefab::ImageBox(self.image_box_to_prefab(data)?)
            }
            WidgetUnitNode::TextBox(data) => {
                WidgetUnitNodePrefab::TextBox(self.text_box_to_prefab(data)?)
            }
        })
    }

    fn area_box_to_prefab(
        &self,
        data: &AreaBoxNode,
    ) -> Result<AreaBoxNodePrefab, ApplicationError> {
        Ok(AreaBoxNodePrefab {
            id: data.id.to_owned(),
            slot: Box::new(self.node_to_prefab(&data.slot)?),
            renderer_effect: data.renderer_effect.to_owned(),
        })
    }

    fn portal_box_to_prefab(
        &self,
        data: &PortalBoxNode,
    ) -> Result<PortalBoxNodePrefab, ApplicationError> {
        Ok(PortalBoxNodePrefab {
            id: data.id.to_owned(),
            slot: Box::new(match &*data.slot {
                PortalBoxSlotNode::Slot(slot) => {
                    PortalBoxSlotNodePrefab::Slot(self.node_to_prefab(&slot)?)
                }
                PortalBoxSlotNode::ContentItem(item) => {
                    PortalBoxSlotNodePrefab::ContentItem(ContentBoxItemNodePrefab {
                        slot: self.node_to_prefab(&item.slot)?,
                        layout: item.layout.clone(),
                    })
                }
                PortalBoxSlotNode::FlexItem(item) => {
                    PortalBoxSlotNodePrefab::FlexItem(FlexBoxItemNodePrefab {
                        slot: self.node_to_prefab(&item.slot)?,
                        layout: item.layout.clone(),
                    })
                }
                PortalBoxSlotNode::GridItem(item) => {
                    PortalBoxSlotNodePrefab::GridItem(GridBoxItemNodePrefab {
                        slot: self.node_to_prefab(&item.slot)?,
                        layout: item.layout.clone(),
                    })
                }
            }),
            owner: data.owner.to_owned(),
        })
    }

    fn content_box_to_prefab(
        &self,
        data: &ContentBoxNode,
    ) -> Result<ContentBoxNodePrefab, ApplicationError> {
        Ok(ContentBoxNodePrefab {
            id: data.id.to_owned(),
            props: self.props_registry.serialize(&data.props)?,
            items: data
                .items
                .iter()
                .map(|v| {
                    Ok(ContentBoxItemNodePrefab {
                        slot: self.node_to_prefab(&v.slot)?,
                        layout: v.layout.clone(),
                    })
                })
                .collect::<Result<_, ApplicationError>>()?,
            clipping: data.clipping,
            transform: data.transform,
        })
    }

    fn flex_box_to_prefab(
        &self,
        data: &FlexBoxNode,
    ) -> Result<FlexBoxNodePrefab, ApplicationError> {
        Ok(FlexBoxNodePrefab {
            id: data.id.to_owned(),
            props: self.props_registry.serialize(&data.props)?,
            items: data
                .items
                .iter()
                .map(|v| {
                    Ok(FlexBoxItemNodePrefab {
                        slot: self.node_to_prefab(&v.slot)?,
                        layout: v.layout.clone(),
                    })
                })
                .collect::<Result<_, ApplicationError>>()?,
            direction: data.direction,
            separation: data.separation,
            wrap: data.wrap,
            transform: data.transform,
        })
    }

    fn grid_box_to_prefab(
        &self,
        data: &GridBoxNode,
    ) -> Result<GridBoxNodePrefab, ApplicationError> {
        Ok(GridBoxNodePrefab {
            id: data.id.to_owned(),
            props: self.props_registry.serialize(&data.props)?,
            items: data
                .items
                .iter()
                .map(|v| {
                    Ok(GridBoxItemNodePrefab {
                        slot: self.node_to_prefab(&v.slot)?,
                        layout: v.layout.clone(),
                    })
                })
                .collect::<Result<_, ApplicationError>>()?,
            cols: data.cols,
            rows: data.rows,
            transform: data.transform,
        })
    }

    fn size_box_to_prefab(
        &self,
        data: &SizeBoxNode,
    ) -> Result<SizeBoxNodePrefab, ApplicationError> {
        Ok(SizeBoxNodePrefab {
            id: data.id.to_owned(),
            props: self.props_registry.serialize(&data.props)?,
            slot: Box::new(self.node_to_prefab(&data.slot)?),
            width: data.width,
            height: data.height,
            margin: data.margin,
            transform: data.transform,
        })
    }

    fn image_box_to_prefab(
        &self,
        data: &ImageBoxNode,
    ) -> Result<ImageBoxNodePrefab, ApplicationError> {
        Ok(ImageBoxNodePrefab {
            id: data.id.to_owned(),
            props: self.props_registry.serialize(&data.props)?,
            width: data.width,
            height: data.height,
            content_keep_aspect_ratio: data.content_keep_aspect_ratio,
            material: data.material.clone(),
            transform: data.transform,
        })
    }

    fn text_box_to_prefab(
        &self,
        data: &TextBoxNode,
    ) -> Result<TextBoxNodePrefab, ApplicationError> {
        Ok(TextBoxNodePrefab {
            id: data.id.to_owned(),
            props: self.props_registry.serialize(&data.props)?,
            text: data.text.clone(),
            width: data.width,
            height: data.height,
            alignment: data.alignment,
            direction: data.direction,
            font: data.font.clone(),
            color: data.color,
            transform: data.transform,
        })
    }

    fn node_from_prefab(&self, data: WidgetNodePrefab) -> Result<WidgetNode, ApplicationError> {
        Ok(match data {
            WidgetNodePrefab::None => WidgetNode::None,
            WidgetNodePrefab::Component(data) => {
                WidgetNode::Component(self.component_from_prefab(data)?)
            }
            WidgetNodePrefab::Unit(data) => WidgetNode::Unit(self.unit_from_prefab(data)?),
        })
    }

    fn component_from_prefab(
        &self,
        data: WidgetComponentPrefab,
    ) -> Result<WidgetComponent, ApplicationError> {
        if let Some(processor) = self.component_mappings.get(&data.type_name) {
            Ok(WidgetComponent {
                processor: *processor,
                type_name: data.type_name,
                key: data.key,
                idref: Default::default(),
                props: self.deserialize_props(data.props)?,
                shared_props: match data.shared_props {
                    Some(p) => Some(self.deserialize_props(p)?),
                    None => None,
                },
                listed_slots: data
                    .listed_slots
                    .into_iter()
                    .map(|v| self.node_from_prefab(v))
                    .collect::<Result<_, ApplicationError>>()?,
                named_slots: data
                    .named_slots
                    .into_iter()
                    .map(|(k, v)| Ok((k, self.node_from_prefab(v)?)))
                    .collect::<Result<_, ApplicationError>>()?,
            })
        } else {
            Err(ApplicationError::ComponentMappingNotFound(
                data.type_name.clone(),
            ))
        }
    }

    fn unit_from_prefab(
        &self,
        data: WidgetUnitNodePrefab,
    ) -> Result<WidgetUnitNode, ApplicationError> {
        Ok(match data {
            WidgetUnitNodePrefab::None => WidgetUnitNode::None,
            WidgetUnitNodePrefab::AreaBox(data) => {
                WidgetUnitNode::AreaBox(self.area_box_from_prefab(data)?)
            }
            WidgetUnitNodePrefab::PortalBox(data) => {
                WidgetUnitNode::PortalBox(self.portal_box_from_prefab(data)?)
            }
            WidgetUnitNodePrefab::ContentBox(data) => {
                WidgetUnitNode::ContentBox(self.content_box_from_prefab(data)?)
            }
            WidgetUnitNodePrefab::FlexBox(data) => {
                WidgetUnitNode::FlexBox(self.flex_box_from_prefab(data)?)
            }
            WidgetUnitNodePrefab::GridBox(data) => {
                WidgetUnitNode::GridBox(self.grid_box_from_prefab(data)?)
            }
            WidgetUnitNodePrefab::SizeBox(data) => {
                WidgetUnitNode::SizeBox(self.size_box_from_prefab(data)?)
            }
            WidgetUnitNodePrefab::ImageBox(data) => {
                WidgetUnitNode::ImageBox(self.image_box_from_prefab(data)?)
            }
            WidgetUnitNodePrefab::TextBox(data) => {
                WidgetUnitNode::TextBox(self.text_box_from_prefab(data)?)
            }
        })
    }

    fn area_box_from_prefab(
        &self,
        data: AreaBoxNodePrefab,
    ) -> Result<AreaBoxNode, ApplicationError> {
        Ok(AreaBoxNode {
            id: data.id,
            slot: Box::new(self.node_from_prefab(*data.slot)?),
            renderer_effect: data.renderer_effect,
        })
    }

    fn portal_box_from_prefab(
        &self,
        data: PortalBoxNodePrefab,
    ) -> Result<PortalBoxNode, ApplicationError> {
        Ok(PortalBoxNode {
            id: data.id,
            slot: Box::new(match *data.slot {
                PortalBoxSlotNodePrefab::Slot(slot) => {
                    PortalBoxSlotNode::Slot(self.node_from_prefab(slot)?)
                }
                PortalBoxSlotNodePrefab::ContentItem(item) => {
                    PortalBoxSlotNode::ContentItem(ContentBoxItemNode {
                        slot: self.node_from_prefab(item.slot)?,
                        layout: item.layout,
                    })
                }
                PortalBoxSlotNodePrefab::FlexItem(item) => {
                    PortalBoxSlotNode::FlexItem(FlexBoxItemNode {
                        slot: self.node_from_prefab(item.slot)?,
                        layout: item.layout,
                    })
                }
                PortalBoxSlotNodePrefab::GridItem(item) => {
                    PortalBoxSlotNode::GridItem(GridBoxItemNode {
                        slot: self.node_from_prefab(item.slot)?,
                        layout: item.layout,
                    })
                }
            }),
            owner: data.owner,
        })
    }

    fn content_box_from_prefab(
        &self,
        data: ContentBoxNodePrefab,
    ) -> Result<ContentBoxNode, ApplicationError> {
        Ok(ContentBoxNode {
            id: data.id,
            props: self.props_registry.deserialize(data.props)?,
            items: data
                .items
                .into_iter()
                .map(|v| {
                    Ok(ContentBoxItemNode {
                        slot: self.node_from_prefab(v.slot)?,
                        layout: v.layout,
                    })
                })
                .collect::<Result<_, ApplicationError>>()?,
            clipping: data.clipping,
            transform: data.transform,
        })
    }

    fn flex_box_from_prefab(
        &self,
        data: FlexBoxNodePrefab,
    ) -> Result<FlexBoxNode, ApplicationError> {
        Ok(FlexBoxNode {
            id: data.id,
            props: self.props_registry.deserialize(data.props)?,
            items: data
                .items
                .into_iter()
                .map(|v| {
                    Ok(FlexBoxItemNode {
                        slot: self.node_from_prefab(v.slot)?,
                        layout: v.layout,
                    })
                })
                .collect::<Result<_, ApplicationError>>()?,
            direction: data.direction,
            separation: data.separation,
            wrap: data.wrap,
            transform: data.transform,
        })
    }

    fn grid_box_from_prefab(
        &self,
        data: GridBoxNodePrefab,
    ) -> Result<GridBoxNode, ApplicationError> {
        Ok(GridBoxNode {
            id: data.id,
            props: self.props_registry.deserialize(data.props)?,
            items: data
                .items
                .into_iter()
                .map(|v| {
                    Ok(GridBoxItemNode {
                        slot: self.node_from_prefab(v.slot)?,
                        layout: v.layout,
                    })
                })
                .collect::<Result<_, ApplicationError>>()?,
            cols: data.cols,
            rows: data.rows,
            transform: data.transform,
        })
    }

    fn size_box_from_prefab(
        &self,
        data: SizeBoxNodePrefab,
    ) -> Result<SizeBoxNode, ApplicationError> {
        Ok(SizeBoxNode {
            id: data.id,
            props: self.props_registry.deserialize(data.props)?,
            slot: Box::new(self.node_from_prefab(*data.slot)?),
            width: data.width,
            height: data.height,
            margin: data.margin,
            transform: data.transform,
        })
    }

    fn image_box_from_prefab(
        &self,
        data: ImageBoxNodePrefab,
    ) -> Result<ImageBoxNode, ApplicationError> {
        Ok(ImageBoxNode {
            id: data.id,
            props: self.props_registry.deserialize(data.props)?,
            width: data.width,
            height: data.height,
            content_keep_aspect_ratio: data.content_keep_aspect_ratio,
            material: data.material,
            transform: data.transform,
        })
    }

    fn text_box_from_prefab(
        &self,
        data: TextBoxNodePrefab,
    ) -> Result<TextBoxNode, ApplicationError> {
        Ok(TextBoxNode {
            id: data.id,
            props: self.props_registry.deserialize(data.props)?,
            text: data.text,
            width: data.width,
            height: data.height,
            alignment: data.alignment,
            direction: data.direction,
            font: data.font,
            color: data.color,
            transform: data.transform,
        })
    }
}
