use crate::{
    animator::{AnimationUpdate, Animator, AnimatorState},
    interactive::InteractionsEngine,
    layout::{CoordsMapping, Layout, LayoutEngine},
    messenger::{MessageReceiver, MessageSender, Messages, Messenger},
    props::{Props, PropsData, PropsDef},
    renderer::Renderer,
    signals::{Signal, SignalReceiver, SignalSender},
    state::{State, StateData, StateUpdate},
    widget::{
        component::{WidgetComponent, WidgetComponentDef},
        context::{WidgetContext, WidgetMountOrChangeContext, WidgetUnmountContext},
        node::{WidgetNode, WidgetNodeDef},
        unit::{
            content::{
                ContentBoxItemNode, ContentBoxItemNodeDef, ContentBoxNode, ContentBoxNodeDef,
            },
            flex::{FlexBoxItemNode, FlexBoxItemNodeDef, FlexBoxNode, FlexBoxNodeDef},
            grid::{GridBoxItemNode, GridBoxItemNodeDef, GridBoxNode, GridBoxNodeDef},
            image::{ImageBoxNode, ImageBoxNodeDef},
            size::{SizeBoxNode, SizeBoxNodeDef},
            text::{TextBoxNode, TextBoxNodeDef},
            WidgetUnit, WidgetUnitNode, WidgetUnitNodeDef,
        },
        FnWidget, WidgetId, WidgetLifeCycle, WidgetUnmountClosure,
    },
    Scalar,
};
use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
    convert::TryInto,
    sync::mpsc::{channel, Receiver, Sender},
};

#[derive(Debug, Clone)]
pub enum ApplicationError {
    PropsMappingByTypeNotFound(TypeId),
    PropsMappingByNameNotFound(String),
    ComponentMappingNotFound(String),
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
    props_mappings: HashMap<TypeId, String>,
    props_mappings_table: HashMap<String, TypeId>,
    tree: WidgetNode,
    rendered_tree: WidgetUnit,
    layout: Layout,
    states: HashMap<WidgetId, StateData>,
    state_receivers: HashMap<WidgetId, Receiver<StateData>>,
    animators: HashMap<WidgetId, AnimatorState>,
    message_sender: MessageSender,
    message_receiver: MessageReceiver,
    signal_sender: Sender<Signal>,
    signal_receiver: SignalReceiver,
    last_signals: Vec<Signal>,
    unmount_closures: HashMap<WidgetId, Vec<Box<WidgetUnmountClosure>>>,
    dirty: bool,
    render_changed: bool,
    last_invalidation_cause: InvalidationCause,
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
        let (message_sender, message_receiver) = channel();
        let message_sender = MessageSender::new(message_sender);
        let message_receiver = MessageReceiver::new(message_receiver);
        let (signal_sender, signal_receiver) = channel();
        let signal_receiver = SignalReceiver::new(signal_receiver);
        Self {
            component_mappings: Default::default(),
            props_mappings: Default::default(),
            props_mappings_table: Default::default(),
            tree: Default::default(),
            rendered_tree: Default::default(),
            layout: Default::default(),
            states: Default::default(),
            state_receivers: Default::default(),
            animators: Default::default(),
            message_sender,
            message_receiver,
            signal_sender,
            signal_receiver,
            last_signals: Default::default(),
            unmount_closures: Default::default(),
            dirty: true,
            render_changed: false,
            last_invalidation_cause: Default::default(),
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
    pub fn map_component(&mut self, type_name: &str, processor: FnWidget) {
        self.component_mappings
            .insert(type_name.to_owned(), processor);
    }

    #[inline]
    pub fn unmap_component(&mut self, type_name: &str) {
        self.component_mappings.remove(type_name);
    }

    #[inline]
    pub fn map_props<T>(&mut self, type_name: &str)
    where
        T: 'static + PropsData,
    {
        let t = TypeId::of::<T>();
        self.props_mappings.insert(t, type_name.to_owned());
        self.props_mappings_table.insert(type_name.to_owned(), t);
    }

    #[inline]
    pub fn unmap_props<T>(&mut self)
    where
        T: 'static + PropsData,
    {
        if let Some(name) = self.props_mappings.remove(&TypeId::of::<T>()) {
            self.props_mappings_table.remove(&name);
        }
    }

    pub fn props_to_serializable(&self, props: Props) -> Result<PropsDef, ApplicationError> {
        let props = props
            .into_inner()
            .into_iter()
            .map(|(k, v)| match self.props_mappings.get(&k) {
                Some(name) => Ok((name.to_owned(), v)),
                None => Err(ApplicationError::PropsMappingByTypeNotFound(k)),
            })
            .collect::<Result<HashMap<_, _>, _>>()?;
        Ok(PropsDef(props))
    }

    pub fn node_to_serializable(
        &self,
        node: WidgetNode,
    ) -> Result<WidgetNodeDef, ApplicationError> {
        Ok(match node {
            WidgetNode::None => WidgetNodeDef::None,
            WidgetNode::Component(v) => {
                WidgetNodeDef::Component(self.component_to_serializable(v)?)
            }
            WidgetNode::Unit(v) => WidgetNodeDef::Unit(self.unit_to_serializable(v)?),
        })
    }

    pub fn component_to_serializable(
        &self,
        component: WidgetComponent,
    ) -> Result<WidgetComponentDef, ApplicationError> {
        let WidgetComponent {
            type_name,
            key,
            props,
            shared_props,
            listed_slots,
            named_slots,
            ..
        } = component;
        if self.component_mappings.get(&type_name).is_none() {
            return Err(ApplicationError::ComponentMappingNotFound(type_name));
        }
        let props = self.props_to_serializable(props)?;
        let shared_props = match shared_props {
            Some(props) => Some(self.props_to_serializable(props)?),
            None => None,
        };
        let listed_slots = listed_slots
            .into_iter()
            .map(|node| self.node_to_serializable(node))
            .collect::<Result<Vec<_>, _>>()?;
        let named_slots = named_slots
            .into_iter()
            .map(|(key, node)| self.node_to_serializable(node).map(|node| (key, node)))
            .collect::<Result<HashMap<_, _>, _>>()?;
        Ok(WidgetComponentDef {
            type_name,
            key,
            props,
            shared_props,
            listed_slots,
            named_slots,
        })
    }

    pub fn unit_to_serializable(
        &self,
        unit: WidgetUnitNode,
    ) -> Result<WidgetUnitNodeDef, ApplicationError> {
        Ok(match unit {
            WidgetUnitNode::None => WidgetUnitNodeDef::None,
            WidgetUnitNode::ContentBox(v) => {
                let ContentBoxNode {
                    id,
                    props,
                    items,
                    clipping,
                    transform,
                } = v;
                let props = self.props_to_serializable(props)?;
                let items = items
                    .into_iter()
                    .map(|item| {
                        let ContentBoxItemNode { slot, layout } = item;
                        self.node_to_serializable(slot)
                            .map(|slot| ContentBoxItemNodeDef { slot, layout })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                WidgetUnitNodeDef::ContentBox(ContentBoxNodeDef {
                    id,
                    props,
                    items,
                    clipping,
                    transform,
                })
            }
            WidgetUnitNode::FlexBox(v) => {
                let FlexBoxNode {
                    id,
                    props,
                    items,
                    direction,
                    separation,
                    wrap,
                    transform,
                } = v;
                let props = self.props_to_serializable(props)?;
                let items = items
                    .into_iter()
                    .map(|item| {
                        let FlexBoxItemNode { slot, layout } = item;
                        self.node_to_serializable(slot)
                            .map(|slot| FlexBoxItemNodeDef { slot, layout })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                WidgetUnitNodeDef::FlexBox(FlexBoxNodeDef {
                    id,
                    props,
                    items,
                    direction,
                    separation,
                    wrap,
                    transform,
                })
            }
            WidgetUnitNode::GridBox(v) => {
                let GridBoxNode {
                    id,
                    props,
                    items,
                    cols,
                    rows,
                    transform,
                } = v;
                let props = self.props_to_serializable(props)?;
                let items = items
                    .into_iter()
                    .map(|item| {
                        let GridBoxItemNode { slot, layout } = item;
                        self.node_to_serializable(slot)
                            .map(|slot| GridBoxItemNodeDef { slot, layout })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                WidgetUnitNodeDef::GridBox(GridBoxNodeDef {
                    id,
                    props,
                    items,
                    cols,
                    rows,
                    transform,
                })
            }
            WidgetUnitNode::SizeBox(v) => {
                let SizeBoxNode {
                    id,
                    props,
                    slot,
                    width,
                    height,
                    margin,
                    transform,
                } = v;
                let props = self.props_to_serializable(props)?;
                let slot = Box::new(self.node_to_serializable(*slot)?);
                WidgetUnitNodeDef::SizeBox(SizeBoxNodeDef {
                    id,
                    props,
                    slot,
                    width,
                    height,
                    margin,
                    transform,
                })
            }
            WidgetUnitNode::ImageBox(v) => {
                let ImageBoxNode {
                    id,
                    props,
                    width,
                    height,
                    content_keep_aspect_ratio,
                    material,
                    transform,
                } = v;
                let props = self.props_to_serializable(props)?;
                WidgetUnitNodeDef::ImageBox(ImageBoxNodeDef {
                    id,
                    props,
                    width,
                    height,
                    content_keep_aspect_ratio,
                    material,
                    transform,
                })
            }
            WidgetUnitNode::TextBox(v) => {
                let TextBoxNode {
                    id,
                    props,
                    text,
                    width,
                    height,
                    alignment,
                    direction,
                    font,
                    color,
                    transform,
                } = v;
                let props = self.props_to_serializable(props)?;
                WidgetUnitNodeDef::TextBox(TextBoxNodeDef {
                    id,
                    props,
                    text,
                    width,
                    height,
                    alignment,
                    direction,
                    font,
                    color,
                    transform,
                })
            }
        })
    }

    pub fn props_from_serializable(&self, props: PropsDef) -> Result<Props, ApplicationError> {
        let props = props
            .0
            .into_iter()
            .map(|(k, v)| match self.props_mappings_table.get(&k) {
                Some(typeid) => Ok((*typeid, v)),
                None => Err(ApplicationError::PropsMappingByNameNotFound(k)),
            })
            .collect::<Result<HashMap<_, _>, _>>()?;
        Ok(Props::from_raw(props))
    }

    pub fn node_from_serializable(
        &self,
        node: WidgetNodeDef,
    ) -> Result<WidgetNode, ApplicationError> {
        Ok(match node {
            WidgetNodeDef::None => WidgetNode::None,
            WidgetNodeDef::Component(v) => {
                WidgetNode::Component(self.component_from_serializable(v)?)
            }
            WidgetNodeDef::Unit(v) => WidgetNode::Unit(self.unit_from_serializable(v)?),
        })
    }

    pub fn component_from_serializable(
        &self,
        component: WidgetComponentDef,
    ) -> Result<WidgetComponent, ApplicationError> {
        let WidgetComponentDef {
            type_name,
            key,
            props,
            shared_props,
            listed_slots,
            named_slots,
        } = component;
        let processor = match self.component_mappings.get(&type_name) {
            Some(processor) => *processor,
            None => return Err(ApplicationError::ComponentMappingNotFound(type_name)),
        };
        let props = self.props_from_serializable(props)?;
        let shared_props = match shared_props {
            Some(props) => Some(self.props_from_serializable(props)?),
            None => None,
        };
        let listed_slots = listed_slots
            .into_iter()
            .map(|node| self.node_from_serializable(node))
            .collect::<Result<Vec<_>, _>>()?;
        let named_slots = named_slots
            .into_iter()
            .map(|(key, node)| self.node_from_serializable(node).map(|node| (key, node)))
            .collect::<Result<HashMap<_, _>, _>>()?;
        Ok(WidgetComponent {
            processor,
            type_name,
            key,
            props,
            shared_props,
            listed_slots,
            named_slots,
        })
    }

    pub fn unit_from_serializable(
        &self,
        unit: WidgetUnitNodeDef,
    ) -> Result<WidgetUnitNode, ApplicationError> {
        Ok(match unit {
            WidgetUnitNodeDef::None => WidgetUnitNode::None,
            WidgetUnitNodeDef::ContentBox(v) => {
                let ContentBoxNodeDef {
                    id,
                    props,
                    items,
                    clipping,
                    transform,
                } = v;
                let props = self.props_from_serializable(props)?;
                let items = items
                    .into_iter()
                    .map(|item| {
                        let ContentBoxItemNodeDef { slot, layout } = item;
                        self.node_from_serializable(slot)
                            .map(|slot| ContentBoxItemNode { slot, layout })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                WidgetUnitNode::ContentBox(ContentBoxNode {
                    id,
                    props,
                    items,
                    clipping,
                    transform,
                })
            }
            WidgetUnitNodeDef::FlexBox(v) => {
                let FlexBoxNodeDef {
                    id,
                    props,
                    items,
                    direction,
                    separation,
                    wrap,
                    transform,
                } = v;
                let props = self.props_from_serializable(props)?;
                let items = items
                    .into_iter()
                    .map(|item| {
                        let FlexBoxItemNodeDef { slot, layout } = item;
                        self.node_from_serializable(slot)
                            .map(|slot| FlexBoxItemNode { slot, layout })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                WidgetUnitNode::FlexBox(FlexBoxNode {
                    id,
                    props,
                    items,
                    direction,
                    separation,
                    wrap,
                    transform,
                })
            }
            WidgetUnitNodeDef::GridBox(v) => {
                let GridBoxNodeDef {
                    id,
                    props,
                    items,
                    cols,
                    rows,
                    transform,
                } = v;
                let props = self.props_from_serializable(props)?;
                let items = items
                    .into_iter()
                    .map(|item| {
                        let GridBoxItemNodeDef { slot, layout } = item;
                        self.node_from_serializable(slot)
                            .map(|slot| GridBoxItemNode { slot, layout })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                WidgetUnitNode::GridBox(GridBoxNode {
                    id,
                    props,
                    items,
                    cols,
                    rows,
                    transform,
                })
            }
            WidgetUnitNodeDef::SizeBox(v) => {
                let SizeBoxNodeDef {
                    id,
                    props,
                    slot,
                    width,
                    height,
                    margin,
                    transform,
                } = v;
                let props = self.props_from_serializable(props)?;
                let slot = Box::new(self.node_from_serializable(*slot)?);
                WidgetUnitNode::SizeBox(SizeBoxNode {
                    id,
                    props,
                    slot,
                    width,
                    height,
                    margin,
                    transform,
                })
            }
            WidgetUnitNodeDef::ImageBox(v) => {
                let ImageBoxNodeDef {
                    id,
                    props,
                    width,
                    height,
                    content_keep_aspect_ratio,
                    material,
                    transform,
                } = v;
                let props = self.props_from_serializable(props)?;
                WidgetUnitNode::ImageBox(ImageBoxNode {
                    id,
                    props,
                    width,
                    height,
                    content_keep_aspect_ratio,
                    material,
                    transform,
                })
            }
            WidgetUnitNodeDef::TextBox(v) => {
                let TextBoxNodeDef {
                    id,
                    props,
                    text,
                    width,
                    height,
                    alignment,
                    direction,
                    font,
                    color,
                    transform,
                } = v;
                let props = self.props_from_serializable(props)?;
                WidgetUnitNode::TextBox(TextBoxNode {
                    id,
                    props,
                    text,
                    width,
                    height,
                    alignment,
                    direction,
                    font,
                    color,
                    transform,
                })
            }
        })
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
    pub fn interact<I, R, E>(&self, interactions_engine: &mut I) -> Result<R, E>
    where
        I: InteractionsEngine<R, E>,
    {
        interactions_engine.perform_interactions(self)
    }

    #[inline]
    pub fn messenger(&self) -> &MessageSender {
        &self.message_sender
    }

    #[inline]
    pub fn signals(&self) -> &[Signal] {
        &self.last_signals
    }

    #[inline]
    pub fn forced_process(&mut self) -> bool {
        self.dirty = true;
        self.process()
    }

    pub fn process(&mut self) -> bool {
        self.animations_delta_time = self.animations_delta_time.max(0.0);
        self.last_invalidation_cause = InvalidationCause::None;
        self.render_changed = false;
        let changed_states = self
            .state_receivers
            .iter()
            .filter_map(|(id, receiver)| {
                receiver.try_iter().last().map(|state| (id.clone(), state))
            })
            .collect::<HashMap<_, _>>();
        let mut messages = self.message_receiver.process();
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
        for (k, a) in &mut self.animators {
            a.process(self.animations_delta_time, &k, &self.message_sender);
        }
        self.dirty = false;
        self.state_receivers.clear();
        let old_states = std::mem::take(&mut self.states);
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
            None,
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
                            let messenger = &self.message_sender;
                            let signals = SignalSender::new(id.clone(), self.signal_sender.clone());
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
        self.last_signals = self.signal_receiver.read_all();
        self.animators = std::mem::take(&mut self.animators)
            .into_iter()
            .filter_map(|(k, a)| if a.in_progress() { Some((k, a)) } else { None })
            .collect::<HashMap<_, _>>();
        if let Ok(tree) = rendered_tree.try_into() {
            self.rendered_tree = tree;
            true
        } else {
            false
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn process_node<'a>(
        &mut self,
        node: WidgetNode,
        states: &'a HashMap<WidgetId, StateData>,
        path: Vec<String>,
        messages: &mut HashMap<WidgetId, Messages>,
        new_states: &mut HashMap<WidgetId, StateData>,
        used_ids: &mut HashSet<WidgetId>,
        possible_key: String,
        master_shared_props: Option<Props>,
    ) -> WidgetNode {
        match node {
            WidgetNode::Component(component) => self.process_node_component(
                component,
                states,
                path,
                messages,
                new_states,
                used_ids,
                possible_key,
                master_shared_props,
            ),
            WidgetNode::Unit(unit) => self.process_node_unit(
                unit,
                states,
                path,
                messages,
                new_states,
                used_ids,
                master_shared_props,
            ),
            _ => node,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn process_node_component<'a>(
        &mut self,
        component: WidgetComponent,
        states: &'a HashMap<WidgetId, StateData>,
        mut path: Vec<String>,
        messages: &mut HashMap<WidgetId, Messages>,
        new_states: &mut HashMap<WidgetId, StateData>,
        used_ids: &mut HashSet<WidgetId>,
        possible_key: String,
        master_shared_props: Option<Props>,
    ) -> WidgetNode {
        let WidgetComponent {
            processor,
            type_name,
            key,
            props,
            shared_props,
            listed_slots,
            named_slots,
        } = component;
        let shared_props = match (master_shared_props, shared_props) {
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
        let id = WidgetId::new(type_name, path.clone());
        used_ids.insert(id.clone());
        let (sender, receiver) = channel();
        let (animation_sender, animation_receiver) = channel();
        let messages_list = match messages.remove(&id) {
            Some(messages) => messages,
            None => Messages::new(),
        };
        let mut life_cycle = WidgetLifeCycle::default();
        let default_animator_state = AnimatorState::default();
        let (new_node, mounted) = match states.get(&id) {
            Some(state) => {
                let state = State::new(state, StateUpdate::new(sender.clone()));
                let animator = self.animators.get(&id).unwrap_or(&default_animator_state);
                let context = WidgetContext {
                    id: &id,
                    key: &key,
                    props: &props,
                    shared_props: &shared_props,
                    state,
                    animator,
                    life_cycle: &mut life_cycle,
                    named_slots,
                    listed_slots,
                };
                ((processor)(context), false)
            }
            None => {
                let state_data = Box::new(()) as StateData;
                let state = State::new(&state_data, StateUpdate::new(sender.clone()));
                let animator = self.animators.get(&id).unwrap_or(&default_animator_state);
                let context = WidgetContext {
                    id: &id,
                    key: &key,
                    props: &props,
                    shared_props: &shared_props,
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
                        let state = State::new(state, StateUpdate::new(sender.clone()));
                        let messenger = Messenger::new(self.message_sender.clone(), &messages_list);
                        let signals = SignalSender::new(id.clone(), self.signal_sender.clone());
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
                    let state = State::new(state, StateUpdate::new(sender.clone()));
                    let messenger = Messenger::new(self.message_sender.clone(), &messages_list);
                    let signals = SignalSender::new(id.clone(), self.signal_sender.clone());
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
        while let Ok(data) = animation_receiver.try_recv() {
            match data {
                Some(data) => {
                    self.animators
                        .insert(id.to_owned(), AnimatorState::new(data));
                }
                None => {
                    self.animators.remove(&id);
                }
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
        );
        self.state_receivers.insert(id, receiver);
        new_node
    }

    #[allow(clippy::too_many_arguments)]
    fn process_node_unit<'a>(
        &mut self,
        mut unit: WidgetUnitNode,
        states: &'a HashMap<WidgetId, StateData>,
        path: Vec<String>,
        messages: &mut HashMap<WidgetId, Messages>,
        new_states: &mut HashMap<WidgetId, StateData>,
        used_ids: &mut HashSet<WidgetId>,
        master_shared_props: Option<Props>,
    ) -> WidgetNode {
        match &mut unit {
            WidgetUnitNode::ContentBox(unit) => {
                let items = std::mem::replace(&mut unit.items, Vec::new());
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
                        );
                        node
                    })
                    .collect::<Vec<_>>();
            }
            WidgetUnitNode::FlexBox(unit) => {
                let items = std::mem::replace(&mut unit.items, Vec::new());
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
                        );
                        node
                    })
                    .collect::<Vec<_>>();
            }
            WidgetUnitNode::GridBox(unit) => {
                let items = std::mem::replace(&mut unit.items, Vec::new());
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
                    "0".to_owned(),
                    master_shared_props,
                ));
            }
            _ => {}
        }
        unit.into()
    }
}
