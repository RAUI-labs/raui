//! Application foundation used to drive the RAUI interface
//!
//! An [`Application`] is the struct that pulls together all the pieces of a RAUI ui such as layout,
//! interaction, animations, etc.
//!
//! In most cases users will not need to manually create and manage an [`Application`]. That will
//! usually be handled by renderer integration crates like [`raui-tesselation-renderer`].
//!
//! [`raui-tesselation-renderer`]: https://docs.rs/raui-tesselation-renderer/
//!
//! You _will_ need to interact with [`Application`] if you are building your own RAUI integration
//! with another renderer or game engine.
//! ```

use crate::{
    Prefab, PrefabError, PrefabValue, Scalar,
    animator::{AnimationUpdate, Animator, AnimatorStates},
    interactive::InteractionsEngine,
    layout::{CoordsMapping, Layout, LayoutEngine},
    messenger::{Message, MessageData, MessageSender, Messages, Messenger},
    props::{Props, PropsData, PropsRegistry},
    renderer::Renderer,
    signals::{Signal, SignalSender},
    state::{State, StateChange, StateUpdate},
    view_model::{ViewModel, ViewModelCollection, ViewModelCollectionView},
    widget::{
        FnWidget, WidgetId, WidgetIdCommon, WidgetLifeCycle,
        component::{
            WidgetComponent, WidgetComponentPrefab, containers::responsive_box::MediaQueryViewModel,
        },
        context::{WidgetContext, WidgetMountOrChangeContext, WidgetUnmountContext},
        node::{WidgetNode, WidgetNodePrefab},
        unit::{
            WidgetUnit, WidgetUnitNode, WidgetUnitNodePrefab,
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
        },
    },
};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    convert::TryInto,
    sync::{
        Arc, RwLock,
        mpsc::{Sender, channel},
    },
};

/// Errors that can occur while interacting with an application
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

/// Indicates the reason that an [`Application`] state was invalidated and had to be re-rendered
///
/// You can get the last invalidation cause of an application using [`last_invalidation_cause`]
///
/// [`last_invalidation_cause`]: Application::last_invalidation_cause
#[derive(Debug, Default, Clone)]
pub enum InvalidationCause {
    /// Application not invalidated
    #[default]
    None,
    /// Application update caused by change in widgets common root.
    CommonRootUpdate(WidgetIdCommon),
}

#[derive(Clone)]
pub struct ChangeNotifier(Arc<RwLock<HashSet<WidgetId>>>);

impl ChangeNotifier {
    pub fn notify(&self, id: WidgetId) {
        if let Ok(mut ids) = self.0.write() {
            ids.insert(id);
        }
    }
}

/// Contains and orchestrates application layout, animations, interactions, etc.
///
/// See the [`application`][self] module for more information and examples.
pub struct Application {
    component_mappings: HashMap<String, FnWidget>,
    props_registry: PropsRegistry,
    tree: WidgetNode,
    rendered_tree: WidgetUnit,
    layout: Layout,
    states: HashMap<WidgetId, Props>,
    state_changes: HashMap<WidgetId, Vec<StateChange>>,
    animators: HashMap<WidgetId, AnimatorStates>,
    messages: HashMap<WidgetId, Messages>,
    pending_stack: Vec<WidgetStackItem>,
    done_stack: Vec<WidgetNode>,
    signals: Vec<Signal>,
    pub view_models: ViewModelCollection,
    changes: ChangeNotifier,
    #[allow(clippy::type_complexity)]
    unmount_closures: HashMap<WidgetId, Vec<Box<dyn FnMut(WidgetUnmountContext) + Send + Sync>>>,
    dirty: WidgetIdCommon,
    render_changed: bool,
    last_invalidation_cause: InvalidationCause,
    /// The amount of time between the last update, used when calculating animation progress
    pub animations_delta_time: Scalar,
}

impl Default for Application {
    fn default() -> Self {
        let mut view_models = ViewModelCollection::default();
        view_models.insert(
            MediaQueryViewModel::VIEW_MODEL.to_string(),
            ViewModel::produce(MediaQueryViewModel::new),
        );
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
            pending_stack: Default::default(),
            done_stack: Default::default(),
            signals: Default::default(),
            view_models,
            changes: ChangeNotifier(Default::default()),
            unmount_closures: Default::default(),
            dirty: Default::default(),
            render_changed: false,
            last_invalidation_cause: Default::default(),
            animations_delta_time: 0.0,
        }
    }
}

impl Application {
    /// Setup the application with a given a setup function
    ///
    /// We need to run the `setup` function for the application to register components and
    /// properties if we want to support serialization of the UI. We pass it a function that will do
    /// the actual registration.
    ///
    /// > **Note:** RAUI will work fine without running any `setup` if UI serialization is not
    /// > required.
    #[inline]
    pub fn setup<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Self),
    {
        (f)(self);
    }

    pub fn notifier(&self) -> ChangeNotifier {
        self.changes.clone()
    }

    /// Register's a component under a string name used when serializing the UI
    ///
    /// This function is often used in [`setup`][Self::setup] functions for registering batches of
    /// components.
    #[inline]
    pub fn register_component(&mut self, type_name: &str, processor: FnWidget) {
        self.component_mappings
            .insert(type_name.to_owned(), processor);
    }

    /// Unregisters a component
    ///
    /// See [`register_component`][Self::register_component]
    #[inline]
    pub fn unregister_component(&mut self, type_name: &str) {
        self.component_mappings.remove(type_name);
    }

    /// Register's a property type under a string name used when serializing the UI
    ///
    /// This function is often used in [`setup`][Self::setup] functions for registering batches of
    /// properties.
    #[inline]
    pub fn register_props<T>(&mut self, name: &str)
    where
        T: 'static + Prefab + PropsData,
    {
        self.props_registry.register_factory::<T>(name);
    }

    /// Unregisters a property type
    ///
    /// See [`register_props`][Self::register_props]
    #[inline]
    pub fn unregister_props(&mut self, name: &str) {
        self.props_registry.unregister_factory(name);
    }

    /// Serialize the given [`Props`] to a [`PrefabValue`]
    #[inline]
    pub fn serialize_props(&self, props: &Props) -> Result<PrefabValue, PrefabError> {
        self.props_registry.serialize(props)
    }

    /// Deserialize [`Props`] from a [`PrefabValue`]
    #[inline]
    pub fn deserialize_props(&self, data: PrefabValue) -> Result<Props, PrefabError> {
        self.props_registry.deserialize(data)
    }

    /// Serialize a [`WidgetNode`] to a [`PrefabValue`]
    #[inline]
    pub fn serialize_node(&self, data: &WidgetNode) -> Result<PrefabValue, ApplicationError> {
        Ok(self.node_to_prefab(data)?.to_prefab()?)
    }

    /// Deserialize a [`WidgetNode`] from a [`PrefabValue`]
    #[inline]
    pub fn deserialize_node(&self, data: PrefabValue) -> Result<WidgetNode, ApplicationError> {
        self.node_from_prefab(WidgetNodePrefab::from_prefab(data)?)
    }

    /// Get the reason that the application state was last invalidated and caused to re-process
    #[inline]
    pub fn last_invalidation_cause(&self) -> &InvalidationCause {
        &self.last_invalidation_cause
    }

    /// Return's common root widget ID of widgets that has to be to be re-processed
    #[inline]
    pub fn dirty(&self) -> &WidgetIdCommon {
        &self.dirty
    }

    /// Force mark the application as needing to re-process its root
    #[inline]
    pub fn mark_dirty(&mut self) {
        self.dirty = WidgetIdCommon::new(WidgetId::empty());
    }

    #[inline]
    pub fn does_render_changed(&self) -> bool {
        self.render_changed
    }

    /// Get the [`WidgetNode`] for the application tree
    #[inline]
    pub fn tree(&self) -> &WidgetNode {
        &self.tree
    }

    /// Get the application widget tree rendered to raw [`WidgetUnit`]'s
    #[inline]
    pub fn rendered_tree(&self) -> &WidgetUnit {
        &self.rendered_tree
    }

    /// Get the application [`Layout`] data
    #[inline]
    pub fn layout_data(&self) -> &Layout {
        &self.layout
    }

    #[inline]
    pub fn has_layout_widget(&self, id: &WidgetId) -> bool {
        self.layout.items.keys().any(|k| k == id)
    }

    /// Update the application widget tree
    #[inline]
    pub fn apply(&mut self, tree: impl Into<WidgetNode>) {
        self.mark_dirty();
        self.tree = tree.into();
    }

    /// Render the application
    #[inline]
    pub fn render<R, T, E>(&self, mapping: &CoordsMapping, renderer: &mut R) -> Result<T, E>
    where
        R: Renderer<T, E>,
    {
        renderer.render(&self.rendered_tree, mapping, &self.layout)
    }

    /// Render the application, but only if something effecting the rendering has changed and it
    /// _needs_ to be re-rendered
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

    /// Calculate application layout
    #[inline]
    pub fn layout<L, E>(&mut self, mapping: &CoordsMapping, layout_engine: &mut L) -> Result<(), E>
    where
        L: LayoutEngine<E>,
    {
        self.layout = layout_engine.layout(mapping, &self.rendered_tree)?;
        if let Some(view_model) = self.view_models.get_mut(MediaQueryViewModel::VIEW_MODEL)
            && let Some(mut view_model) = view_model.write::<MediaQueryViewModel>()
        {
            view_model
                .screen_size
                .set_unique_notify(self.layout.ui_space.size());
        }
        Ok(())
    }

    /// Calculate application layout, but only if something effecting application layout has changed
    /// and the layout _needs_ to be re-done
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

    /// Perform interactions on the application using the given interaction engine
    #[inline]
    pub fn interact<I, R, E>(&mut self, interactions_engine: &mut I) -> Result<R, E>
    where
        I: InteractionsEngine<R, E>,
    {
        interactions_engine.perform_interactions(self)
    }

    /// Send a message to the given widget
    #[inline]
    pub fn send_message<T>(&mut self, id: &WidgetId, data: T)
    where
        T: 'static + MessageData,
    {
        self.send_message_raw(id, Box::new(data));
    }

    /// Send raw message data to the given widget
    #[inline]
    pub fn send_message_raw(&mut self, id: &WidgetId, data: Message) {
        if let Some(list) = self.messages.get_mut(id) {
            list.push(data);
        } else {
            self.messages.insert(id.to_owned(), vec![data]);
        }
    }

    /// Get the list of [signals][crate::signals] that have been sent by widgets
    #[inline]
    pub fn signals(&self) -> &[Signal] {
        &self.signals
    }

    /// Get the list of [signals][crate::signals] that have been sent by widgets, consuming the
    /// current list so that further calls will not include previously sent signals
    #[inline]
    pub fn consume_signals(&mut self) -> Vec<Signal> {
        std::mem::take(&mut self.signals)
    }

    /// [`process()`][Self::process] application, even if no changes have been detected
    #[inline]
    pub fn forced_process(&mut self) -> bool {
        self.mark_dirty();
        self.process()
    }

    /// [Process][Self::process] the application.
    pub fn process(&mut self) -> bool {
        self.dirty
            .include_other(&self.view_models.consume_notified_common_root());
        if let Ok(mut ids) = self.changes.0.write() {
            for id in ids.drain() {
                self.dirty.include(&id);
            }
        }
        self.animations_delta_time = self.animations_delta_time.max(0.0);
        self.last_invalidation_cause = InvalidationCause::None;
        self.render_changed = false;
        let changed_states = std::mem::take(&mut self.state_changes);
        for id in changed_states.keys() {
            self.dirty.include(id);
        }
        let mut messages = std::mem::take(&mut self.messages);
        for id in messages.keys() {
            self.dirty.include(id);
        }
        for (id, animator) in &self.animators {
            if animator.in_progress() {
                self.dirty.include(id);
            }
        }
        if !self.dirty.is_valid() {
            return false;
        }
        self.last_invalidation_cause = InvalidationCause::CommonRootUpdate(self.dirty.to_owned());
        let (message_sender, message_receiver) = channel();
        let message_sender = MessageSender::new(message_sender);
        for (k, a) in &mut self.animators {
            a.process(self.animations_delta_time, k, &message_sender);
        }
        let mut states = std::mem::take(&mut self.states);
        for (id, changes) in changed_states {
            let state = states.entry(id).or_default();
            for change in changes {
                match change {
                    StateChange::Set(props) => {
                        *state = props;
                    }
                    StateChange::Include(props) => {
                        state.merge_from(props);
                    }
                    StateChange::Exclude(type_id) => unsafe {
                        state.remove_by_type(type_id);
                    },
                }
            }
        }
        let (signal_sender, signal_receiver) = channel();
        let tree = self.tree.clone();
        let mut used_ids = HashSet::new();
        let mut new_states = HashMap::new();
        let rendered_tree = self.process_nodes_stack(
            tree,
            &states,
            &mut messages,
            &mut new_states,
            &mut used_ids,
            &message_sender,
            &signal_sender,
        );
        self.states = states
            .into_iter()
            .chain(new_states)
            .filter(|(id, state)| {
                if used_ids.contains(id) {
                    true
                } else {
                    if let Some(closures) = self.unmount_closures.remove(id) {
                        for mut closure in closures {
                            let messenger = &message_sender;
                            let signals = SignalSender::new(id.clone(), signal_sender.clone());
                            let view_models =
                                ViewModelCollectionView::new(id, &mut self.view_models);
                            let context = WidgetUnmountContext {
                                id,
                                state,
                                messenger,
                                signals,
                                view_models,
                            };
                            (closure)(context);
                        }
                    }
                    self.animators.remove(id);
                    self.view_models.unbind_all(id);
                    self.view_models.remove_widget_view_models(id);
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
            .collect();
        self.dirty = Default::default();
        if let Ok(tree) = rendered_tree.try_into() {
            self.rendered_tree = Self::teleport_portals(tree);
            true
        } else {
            false
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn process_nodes_stack(
        &mut self,
        root_node: WidgetNode,
        states: &HashMap<WidgetId, Props>,
        messages: &mut HashMap<WidgetId, Messages>,
        new_states: &mut HashMap<WidgetId, Props>,
        used_ids: &mut HashSet<WidgetId>,
        message_sender: &MessageSender,
        signal_sender: &Sender<Signal>,
    ) -> WidgetNode {
        self.pending_stack.clear();
        self.pending_stack.push(WidgetStackItem::Node {
            node: root_node,
            path: vec![],
            possible_key: "<*>".to_string(),
            master_shared_props: None,
        });
        self.done_stack.clear();
        while let Some(item) = self.pending_stack.pop() {
            match item {
                WidgetStackItem::Node {
                    node,
                    mut path,
                    possible_key,
                    master_shared_props,
                } => match node {
                    WidgetNode::None | WidgetNode::Tuple(_) => {
                        self.done_stack.push(node);
                    }
                    WidgetNode::Component(component) => {
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
                        path.push(key.clone().into());
                        let id = WidgetId::new(&type_name, &path);
                        used_ids.insert(id.clone());
                        if let Some(idref) = &mut idref {
                            idref.write(id.to_owned());
                        }
                        let (state_sender, state_receiver) = channel();
                        let (animation_sender, animation_receiver) = channel();
                        let messages_list = messages.remove(&id).unwrap_or_default();
                        let mut life_cycle = WidgetLifeCycle::default();
                        let default_animator_state = AnimatorStates::default();
                        let (new_node, mounted) = match states.get(&id) {
                            Some(state) => {
                                let state =
                                    State::new(state, StateUpdate::new(state_sender.clone()));
                                let animator =
                                    self.animators.get(&id).unwrap_or(&default_animator_state);
                                let view_models =
                                    ViewModelCollectionView::new(&id, &mut self.view_models);
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
                                    view_models,
                                };
                                (processor.call(context), false)
                            }
                            None => {
                                let state_data = Props::default();
                                let state =
                                    State::new(&state_data, StateUpdate::new(state_sender.clone()));
                                let animator =
                                    self.animators.get(&id).unwrap_or(&default_animator_state);
                                let view_models =
                                    ViewModelCollectionView::new(&id, &mut self.view_models);
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
                                    view_models,
                                };
                                let node = processor.call(context);
                                new_states.insert(id.clone(), state_data);
                                (node, true)
                            }
                        };
                        let (mount, change, unmount) = life_cycle.unwrap();
                        if mounted {
                            if !mount.is_empty()
                                && let Some(state) = new_states.get(&id)
                            {
                                for mut closure in mount {
                                    let state =
                                        State::new(state, StateUpdate::new(state_sender.clone()));
                                    let messenger =
                                        Messenger::new(message_sender.clone(), &messages_list);
                                    let signals =
                                        SignalSender::new(id.clone(), signal_sender.clone());
                                    let animator = Animator::new(
                                        self.animators.get(&id).unwrap_or(&default_animator_state),
                                        AnimationUpdate::new(animation_sender.clone()),
                                    );
                                    let view_models =
                                        ViewModelCollectionView::new(&id, &mut self.view_models);
                                    let context = WidgetMountOrChangeContext {
                                        id: &id,
                                        props: &props,
                                        shared_props: &shared_props,
                                        state,
                                        messenger,
                                        signals,
                                        animator,
                                        view_models,
                                    };
                                    (closure)(context);
                                }
                            }
                        } else if !change.is_empty()
                            && let Some(state) = states.get(&id)
                        {
                            for mut closure in change {
                                let state =
                                    State::new(state, StateUpdate::new(state_sender.clone()));
                                let messenger =
                                    Messenger::new(message_sender.clone(), &messages_list);
                                let signals = SignalSender::new(id.clone(), signal_sender.clone());
                                let animator = Animator::new(
                                    self.animators.get(&id).unwrap_or(&default_animator_state),
                                    AnimationUpdate::new(animation_sender.clone()),
                                );
                                let view_models =
                                    ViewModelCollectionView::new(&id, &mut self.view_models);
                                let context = WidgetMountOrChangeContext {
                                    id: &id,
                                    props: &props,
                                    shared_props: &shared_props,
                                    state,
                                    messenger,
                                    signals,
                                    animator,
                                    view_models,
                                };
                                (closure)(context);
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
                        while let Ok(data) = state_receiver.try_recv() {
                            self.state_changes
                                .entry(id.to_owned())
                                .or_default()
                                .push(data);
                        }
                        self.pending_stack.push(WidgetStackItem::Node {
                            node: new_node,
                            path,
                            possible_key,
                            master_shared_props: Some(shared_props),
                        });
                    }
                    WidgetNode::Unit(unit) => match unit {
                        WidgetUnitNode::None
                        | WidgetUnitNode::ImageBox(_)
                        | WidgetUnitNode::TextBox(_) => {
                            self.done_stack.push(WidgetNode::Unit(unit));
                        }
                        WidgetUnitNode::AreaBox(mut unit) => {
                            let slot = *std::mem::take(&mut unit.slot);
                            self.pending_stack
                                .push(WidgetStackItem::AreaBox { node: unit });
                            self.pending_stack.push(WidgetStackItem::Node {
                                node: slot,
                                path,
                                possible_key: ".".to_owned(),
                                master_shared_props,
                            });
                        }
                        WidgetUnitNode::PortalBox(mut unit) => match &mut *unit.slot {
                            PortalBoxSlotNode::Slot(data) => {
                                let slot = std::mem::take(data);
                                self.pending_stack
                                    .push(WidgetStackItem::PortalBox { node: unit });
                                self.pending_stack.push(WidgetStackItem::Node {
                                    node: slot,
                                    path,
                                    possible_key: ".".to_owned(),
                                    master_shared_props,
                                });
                            }
                            PortalBoxSlotNode::ContentItem(item) => {
                                let slot = std::mem::take(&mut item.slot);
                                self.pending_stack
                                    .push(WidgetStackItem::PortalBox { node: unit });
                                self.pending_stack.push(WidgetStackItem::Node {
                                    node: slot,
                                    path,
                                    possible_key: ".".to_owned(),
                                    master_shared_props,
                                });
                            }
                            PortalBoxSlotNode::FlexItem(item) => {
                                let slot = std::mem::take(&mut item.slot);
                                self.pending_stack
                                    .push(WidgetStackItem::PortalBox { node: unit });
                                self.pending_stack.push(WidgetStackItem::Node {
                                    node: slot,
                                    path,
                                    possible_key: ".".to_owned(),
                                    master_shared_props,
                                });
                            }
                            PortalBoxSlotNode::GridItem(item) => {
                                let slot = std::mem::take(&mut item.slot);
                                self.pending_stack
                                    .push(WidgetStackItem::PortalBox { node: unit });
                                self.pending_stack.push(WidgetStackItem::Node {
                                    node: slot,
                                    path,
                                    possible_key: ".".to_owned(),
                                    master_shared_props,
                                });
                            }
                        },
                        WidgetUnitNode::ContentBox(mut unit) => {
                            let items = unit
                                .items
                                .iter_mut()
                                .map(|node| std::mem::take(&mut node.slot))
                                .collect::<Vec<_>>();
                            self.pending_stack
                                .push(WidgetStackItem::ContentBox { node: unit });
                            for (index, node) in items.into_iter().enumerate() {
                                self.pending_stack.push(WidgetStackItem::Node {
                                    node,
                                    path: path.clone(),
                                    possible_key: format!("<{index}>"),
                                    master_shared_props: master_shared_props.clone(),
                                });
                            }
                        }
                        WidgetUnitNode::FlexBox(mut unit) => {
                            let items = unit
                                .items
                                .iter_mut()
                                .map(|node| std::mem::take(&mut node.slot))
                                .collect::<Vec<_>>();
                            self.pending_stack
                                .push(WidgetStackItem::FlexBox { node: unit });
                            for (index, node) in items.into_iter().enumerate() {
                                self.pending_stack.push(WidgetStackItem::Node {
                                    node,
                                    path: path.clone(),
                                    possible_key: format!("<{index}>"),
                                    master_shared_props: master_shared_props.clone(),
                                });
                            }
                        }
                        WidgetUnitNode::GridBox(mut unit) => {
                            let items = unit
                                .items
                                .iter_mut()
                                .map(|node| std::mem::take(&mut node.slot))
                                .collect::<Vec<_>>();
                            self.pending_stack
                                .push(WidgetStackItem::GridBox { node: unit });
                            for (index, node) in items.into_iter().enumerate() {
                                self.pending_stack.push(WidgetStackItem::Node {
                                    node,
                                    path: path.clone(),
                                    possible_key: format!("<{index}>"),
                                    master_shared_props: master_shared_props.clone(),
                                });
                            }
                        }
                        WidgetUnitNode::SizeBox(mut unit) => {
                            let slot = *std::mem::take(&mut unit.slot);
                            self.pending_stack
                                .push(WidgetStackItem::SizeBox { node: unit });
                            self.pending_stack.push(WidgetStackItem::Node {
                                node: slot,
                                path,
                                possible_key: ".".to_owned(),
                                master_shared_props,
                            });
                        }
                    },
                },
                WidgetStackItem::AreaBox { mut node } => {
                    node.slot = Box::new(self.done_stack.pop().unwrap_or_default());
                    self.done_stack
                        .push(WidgetNode::Unit(WidgetUnitNode::AreaBox(node)));
                }
                WidgetStackItem::PortalBox { mut node } => {
                    match &mut *node.slot {
                        PortalBoxSlotNode::Slot(node) => {
                            *node = self.done_stack.pop().unwrap_or_default();
                        }
                        PortalBoxSlotNode::ContentItem(node) => {
                            node.slot = self.done_stack.pop().unwrap_or_default();
                        }
                        PortalBoxSlotNode::FlexItem(node) => {
                            node.slot = self.done_stack.pop().unwrap_or_default();
                        }
                        PortalBoxSlotNode::GridItem(node) => {
                            node.slot = self.done_stack.pop().unwrap_or_default();
                        }
                    }
                    self.done_stack
                        .push(WidgetNode::Unit(WidgetUnitNode::PortalBox(node)));
                }
                WidgetStackItem::ContentBox { mut node } => {
                    for item in node.items.iter_mut() {
                        item.slot = self.done_stack.pop().unwrap_or_default();
                    }
                    self.done_stack
                        .push(WidgetNode::Unit(WidgetUnitNode::ContentBox(node)));
                }
                WidgetStackItem::FlexBox { mut node } => {
                    for item in node.items.iter_mut() {
                        item.slot = self.done_stack.pop().unwrap_or_default();
                    }
                    self.done_stack
                        .push(WidgetNode::Unit(WidgetUnitNode::FlexBox(node)));
                }
                WidgetStackItem::GridBox { mut node } => {
                    for item in node.items.iter_mut() {
                        item.slot = self.done_stack.pop().unwrap_or_default();
                    }
                    self.done_stack
                        .push(WidgetNode::Unit(WidgetUnitNode::GridBox(node)));
                }
                WidgetStackItem::SizeBox { mut node } => {
                    node.slot = Box::new(self.done_stack.pop().unwrap_or_default());
                    self.done_stack
                        .push(WidgetNode::Unit(WidgetUnitNode::SizeBox(node)));
                }
            }
        }
        assert!(
            self.pending_stack.is_empty(),
            "Pending stack should be empty after processing"
        );
        assert_eq!(
            self.done_stack.len(),
            1,
            "Done stack should have exactly one item after processing"
        );
        self.done_stack.pop().unwrap_or_default()
    }

    fn teleport_portals(mut root: WidgetUnit) -> WidgetUnit {
        let count = Self::estimate_portals(&root);
        if count == 0 {
            return root;
        }
        let mut portals = Vec::with_capacity(count);
        Self::consume_portals(&mut root, &mut portals);
        Self::inject_portals(&mut root, &mut portals);
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

    fn inject_portals(unit: &mut WidgetUnit, portals: &mut Vec<(WidgetId, PortalBoxSlot)>) -> bool {
        if portals.is_empty() {
            return false;
        }
        while let Some(data) = unit.as_data() {
            let found = portals.iter().position(|(id, _)| data.id() == id);
            if let Some(index) = found {
                let slot = portals.swap_remove(index).1;
                match unit {
                    WidgetUnit::None
                    | WidgetUnit::PortalBox(_)
                    | WidgetUnit::ImageBox(_)
                    | WidgetUnit::TextBox(_) => {}
                    WidgetUnit::AreaBox(b) => {
                        match slot {
                            PortalBoxSlot::Slot(slot) => b.slot = Box::new(slot),
                            PortalBoxSlot::ContentItem(item) => b.slot = Box::new(item.slot),
                            PortalBoxSlot::FlexItem(item) => b.slot = Box::new(item.slot),
                            PortalBoxSlot::GridItem(item) => b.slot = Box::new(item.slot),
                        }
                        if !Self::inject_portals(&mut b.slot, portals) {
                            return false;
                        }
                    }
                    WidgetUnit::ContentBox(b) => {
                        b.items.push(match slot {
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
                        });
                        for item in &mut b.items {
                            if !Self::inject_portals(&mut item.slot, portals) {
                                return false;
                            }
                        }
                    }
                    WidgetUnit::FlexBox(b) => {
                        b.items.push(match slot {
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
                        });
                        for item in &mut b.items {
                            if !Self::inject_portals(&mut item.slot, portals) {
                                return false;
                            }
                        }
                    }
                    WidgetUnit::GridBox(b) => {
                        b.items.push(match slot {
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
                        });
                        for item in &mut b.items {
                            if !Self::inject_portals(&mut item.slot, portals) {
                                return false;
                            }
                        }
                    }
                    WidgetUnit::SizeBox(b) => {
                        match slot {
                            PortalBoxSlot::Slot(slot) => b.slot = Box::new(slot),
                            PortalBoxSlot::ContentItem(item) => b.slot = Box::new(item.slot),
                            PortalBoxSlot::FlexItem(item) => b.slot = Box::new(item.slot),
                            PortalBoxSlot::GridItem(item) => b.slot = Box::new(item.slot),
                        }
                        if !Self::inject_portals(&mut b.slot, portals) {
                            return false;
                        }
                    }
                }
            } else {
                break;
            }
        }
        true
    }

    fn node_to_prefab(&self, data: &WidgetNode) -> Result<WidgetNodePrefab, ApplicationError> {
        Ok(match data {
            WidgetNode::None => WidgetNodePrefab::None,
            WidgetNode::Component(data) => {
                WidgetNodePrefab::Component(self.component_to_prefab(data)?)
            }
            WidgetNode::Unit(data) => WidgetNodePrefab::Unit(self.unit_to_prefab(data)?),
            WidgetNode::Tuple(data) => WidgetNodePrefab::Tuple(self.tuple_to_prefab(data)?),
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

    fn tuple_to_prefab(
        &self,
        data: &[WidgetNode],
    ) -> Result<Vec<WidgetNodePrefab>, ApplicationError> {
        data.iter()
            .map(|node| self.node_to_prefab(node))
            .collect::<Result<_, _>>()
    }

    fn area_box_to_prefab(
        &self,
        data: &AreaBoxNode,
    ) -> Result<AreaBoxNodePrefab, ApplicationError> {
        Ok(AreaBoxNodePrefab {
            id: data.id.to_owned(),
            slot: Box::new(self.node_to_prefab(&data.slot)?),
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
                    PortalBoxSlotNodePrefab::Slot(self.node_to_prefab(slot)?)
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
            content_reposition: data.content_reposition,
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
            keep_aspect_ratio: data.keep_aspect_ratio,
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
            horizontal_align: data.horizontal_align,
            vertical_align: data.vertical_align,
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
            WidgetNodePrefab::Tuple(data) => WidgetNode::Tuple(self.tuple_from_prefab(data)?),
        })
    }

    fn component_from_prefab(
        &self,
        data: WidgetComponentPrefab,
    ) -> Result<WidgetComponent, ApplicationError> {
        if let Some(processor) = self.component_mappings.get(&data.type_name) {
            Ok(WidgetComponent {
                processor: processor.clone(),
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

    fn tuple_from_prefab(
        &self,
        data: Vec<WidgetNodePrefab>,
    ) -> Result<Vec<WidgetNode>, ApplicationError> {
        data.into_iter()
            .map(|data| self.node_from_prefab(data))
            .collect::<Result<_, _>>()
    }

    fn area_box_from_prefab(
        &self,
        data: AreaBoxNodePrefab,
    ) -> Result<AreaBoxNode, ApplicationError> {
        Ok(AreaBoxNode {
            id: data.id,
            slot: Box::new(self.node_from_prefab(*data.slot)?),
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
            content_reposition: data.content_reposition,
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
            keep_aspect_ratio: data.keep_aspect_ratio,
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
            horizontal_align: data.horizontal_align,
            vertical_align: data.vertical_align,
            direction: data.direction,
            font: data.font,
            color: data.color,
            transform: data.transform,
        })
    }
}

#[allow(clippy::large_enum_variant)]
enum WidgetStackItem {
    Node {
        node: WidgetNode,
        path: Vec<Cow<'static, str>>,
        possible_key: String,
        master_shared_props: Option<Props>,
    },
    AreaBox {
        node: AreaBoxNode,
    },
    PortalBox {
        node: PortalBoxNode,
    },
    ContentBox {
        node: ContentBoxNode,
    },
    FlexBox {
        node: FlexBoxNode,
    },
    GridBox {
        node: GridBoxNode,
    },
    SizeBox {
        node: SizeBoxNode,
    },
}
