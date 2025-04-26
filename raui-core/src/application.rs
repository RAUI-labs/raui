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
//!
//! # Example
//!
//! ```rust
//! # use raui_core::prelude::*;
//! // Create the application
//! let mut application = Application::default();
//!
//! // We need to run the "setup" functions for the application to register components and
//! // properties if we want to support serialization of the UI. We pass it a function that
//! // will do the actual registration
//! application.setup(setup /* the core setup function from the RAUI prelude */);
//!
//! // If we used RAUI material we would also want to call it's setup ( but we don't need
//! // it here )
//! // application.setup(raui_material::setup);
//!
//! // Create the renderer. In this case we use the raw renderer that will return raw
//! // [`WidgetUnit`]'s, but usually you would have a custom renderer for your game
//! // engine or renderer.
//! let mut renderer = RawRenderer;
//!
//! // Create the interactions engine. The default interactions engine covers typical
//! // pointer + keyboard + gamepad navigation/interactions.
//! let mut interactions = DefaultInteractionsEngine::default();
//!
//! // We create our widget tree
//! let tree = make_widget!(nav_content_box)
//!     .key("app")
//!     .listed_slot(make_widget!(button)
//!         .key("button")
//!         .with_props(NavItemActive)
//!         .named_slot("content", make_widget!(image_box).key("icon"))
//!     );
//!
//! // We apply the tree to the application. This must be done again if we wish to change the
//! // tree.
//! application.apply(tree);
//!
//! // This and the following function calls would need to be called every frame
//! loop {
//!     // Telling the app to `process` will make it perform any necessary updates.
//!     application.process();
//!
//!     // To properly handle layout we need to create a mapping of the screen coordinates to
//!     // the RAUI coordinates. We would update this with the size of the window every frame.
//!     let mapping = CoordsMapping::new(Rect {
//!         left: 0.0,
//!         right: 1024.0,
//!         top: 0.0,
//!         bottom: 576.0,
//!     });
//!
//!     // We apply the application layout
//!     let mut layout_engine = DefaultLayoutEngine::<()>::default();
//!     application
//!         // We use the default layout engine, but you could make your own layout engine
//!         .layout(&mapping, &mut layout_engine)
//!         .unwrap();
//!
//!     // we interact with UI by sending interaction messages to the engine. You would hook this
//!     // up to whatever game engine or window event loop to perform the proper interactions when
//!     // different events are emitted.
//!     interactions.interact(Interaction::PointerMove(Vec2 { x: 200.0, y: 100.0 }));
//!     interactions.interact(Interaction::PointerDown(
//!         PointerButton::Trigger,
//!         Vec2 { x: 200.0, y: 100.0 },
//!     ));
//!
//!     // Since interactions engines require constructed layout to process interactions we
//!     // have to process interactions after we layout the UI.
//!     application.interact(&mut interactions).unwrap();
//!
//!     // Now we render the app, printing it's raw widget units
//!     println!("{:?}", application.render(&mapping, &mut renderer).unwrap());
//! #   break;
//! }
//! ```

use crate::{
    Prefab, PrefabError, PrefabValue, Scalar,
    animator::{AnimationUpdate, Animator, AnimatorStates},
    interactive::InteractionsEngine,
    layout::{CoordsMapping, Layout, LayoutEngine},
    messenger::{Message, MessageData, MessageSender, Messages, Messenger},
    prelude::ViewModelCollectionView,
    props::{Props, PropsData, PropsRegistry},
    renderer::Renderer,
    signals::{Signal, SignalSender},
    state::{State, StateChange, StateUpdate},
    view_model::ViewModelCollection,
    widget::{
        FnWidget, WidgetId, WidgetIdCommon, WidgetLifeCycle,
        component::{WidgetComponent, WidgetComponentPrefab},
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
            view_models: Default::default(),
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
    ///
    /// # Example
    ///
    /// ```
    /// # use raui_core::prelude::*;
    /// # let mut application = Application::default();
    /// application.setup(setup /* the core setup function from the RAUI prelude */);
    /// ```
    ///
    /// If you use crates like the `raui_material` crate you will want to call it's setup function
    /// as well.
    ///
    /// ```ignore
    /// application.setup(raui_material::setup);
    /// ```
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
    ///
    /// # Example
    ///
    /// ```
    /// # use raui_core::prelude::*;
    /// fn my_widget(ctx: WidgetContext) -> WidgetNode {
    ///     todo!("make awesome widget");
    /// }
    ///
    /// fn setup_widgets(app: &mut Application) {
    ///     app.register_component("my_widget", FnWidget::pointer(my_widget));
    /// }
    ///
    /// let mut application = Application::default();
    ///
    /// application.setup(setup_widgets);
    /// ```
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
    ///
    /// # Example
    ///
    /// ```
    /// # use raui_core::prelude::*;
    /// # use serde::{Serialize, Deserialize};
    /// #[derive(PropsData, Debug, Default, Copy, Clone, Serialize, Deserialize)]
    /// struct MyProp {
    ///     awesome: bool,
    /// }
    ///
    /// fn setup_properties(app: &mut Application) {
    ///     app.register_props::<MyProp>("MyProp");
    /// }
    ///
    /// let mut application = Application::default();
    ///
    /// application.setup(setup_properties);
    /// ```
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
    ///
    /// ## Example
    ///
    /// ```
    /// # use raui_core::prelude::*;
    /// const APP_DATA: &str = "app-data";
    /// const COUNTER: &str = "counter";
    ///
    /// /// Some sort of application data.
    /// struct AppData {
    ///     counter: ViewModelValue<i32>,
    /// }
    ///
    /// // Make view-model of that data.
    /// let mut view_model = ViewModel::produce(|properties| {
    ///     AppData {
    ///         counter: ViewModelValue::new(0, properties.notifier(COUNTER)),
    ///     }
    /// });
    ///
    /// // Get handle to view-model data for access on host side.
    /// // This handle is valid as long as it's view-model is alive.
    /// let mut app_data = view_model.lazy::<AppData>().unwrap();
    ///
    /// let mut app = Application::default();
    /// app.view_models.insert(APP_DATA.to_owned(), view_model);
    /// // Do application stuff like interactions, layout, etc...
    ///
    /// // Now we call `process` with our process context
    /// app.process();
    /// ```
    ///
    /// Now, in our components we can access the `AppData` from view-model
    /// through the widget's `WidgetContext`.
    ///
    /// ```
    /// # use raui_core::prelude::*;
    /// # struct AppData {
    /// #    counter: ViewModelValue<i32>,
    /// # }
    /// # const APP_DATA: &str = "app-data";
    /// # const COUNTER: &str = "counter";
    /// fn my_component(mut ctx: WidgetContext) -> WidgetNode {
    ///     let mut data = ctx
    ///         .view_models
    ///         .view_model_mut(APP_DATA)
    ///         .unwrap()
    ///         .write::<AppData>()
    ///         .unwrap();
    ///     *data.counter += 1;
    ///
    ///     // widget stuff...
    /// #   Default::default()
    /// }
    /// ```
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
    fn process_node(
        &mut self,
        node: WidgetNode,
        states: &HashMap<WidgetId, Props>,
        path: Vec<Cow<'static, str>>,
        messages: &mut HashMap<WidgetId, Messages>,
        new_states: &mut HashMap<WidgetId, Props>,
        used_ids: &mut HashSet<WidgetId>,
        possible_key: String,
        master_shared_props: Option<Props>,
        message_sender: &MessageSender,
        signal_sender: &Sender<Signal>,
    ) -> WidgetNode {
        match node {
            WidgetNode::None | WidgetNode::Tuple(_) => node,
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
    fn process_node_component(
        &mut self,
        component: WidgetComponent,
        states: &HashMap<WidgetId, Props>,
        mut path: Vec<Cow<'static, str>>,
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
                let state = State::new(state, StateUpdate::new(state_sender.clone()));
                let animator = self.animators.get(&id).unwrap_or(&default_animator_state);
                let view_models = ViewModelCollectionView::new(&id, &mut self.view_models);
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
                let state = State::new(&state_data, StateUpdate::new(state_sender.clone()));
                let animator = self.animators.get(&id).unwrap_or(&default_animator_state);
                let view_models = ViewModelCollectionView::new(&id, &mut self.view_models);
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
                        let view_models = ViewModelCollectionView::new(&id, &mut self.view_models);
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
                    let view_models = ViewModelCollectionView::new(&id, &mut self.view_models);
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
            self.state_changes
                .entry(id.to_owned())
                .or_default()
                .push(data);
        }
        new_node
    }

    #[allow(clippy::too_many_arguments)]
    fn process_node_unit(
        &mut self,
        mut unit: WidgetUnitNode,
        states: &HashMap<WidgetId, Props>,
        path: Vec<Cow<'static, str>>,
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
