use crate::{
    PropsData, make_widget, pre_hooks, unpack_named_slots,
    view_model::ViewModelValue,
    widget::{
        WidgetIdMetaParams,
        component::{
            containers::{
                anchor_box::PivotBoxProps,
                context_box::{ContextBoxProps, portals_context_box},
                size_box::{SizeBoxProps, size_box},
            },
            interactive::{
                button::{ButtonNotifyMessage, ButtonNotifyProps, button},
                navigation::NavItemActive,
            },
        },
        context::WidgetContext,
        node::WidgetNode,
    },
};
use intuicio_data::managed::ManagedLazy;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

pub trait OptionsViewProxy: Send + Sync {
    fn get(&self) -> usize;
    fn set(&mut self, value: usize);
}

macro_rules! impl_proxy {
    ($type:ty) => {
        impl OptionsViewProxy for $type {
            fn get(&self) -> usize {
                *self as _
            }

            fn set(&mut self, value: usize) {
                *self = value as _;
            }
        }
    };
}

impl_proxy!(u8);
impl_proxy!(u16);
impl_proxy!(u32);
impl_proxy!(u64);
impl_proxy!(u128);
impl_proxy!(usize);
impl_proxy!(i8);
impl_proxy!(i16);
impl_proxy!(i32);
impl_proxy!(i64);
impl_proxy!(i128);
impl_proxy!(isize);
impl_proxy!(f32);
impl_proxy!(f64);

impl<T> OptionsViewProxy for ViewModelValue<T>
where
    T: OptionsViewProxy,
{
    fn get(&self) -> usize {
        self.deref().get()
    }

    fn set(&mut self, value: usize) {
        self.deref_mut().set(value);
    }
}

#[derive(Clone)]
pub struct OptionsInput(ManagedLazy<dyn OptionsViewProxy>);

impl OptionsInput {
    pub fn new(data: ManagedLazy<impl OptionsViewProxy + 'static>) -> Self {
        let (lifetime, data) = data.into_inner();
        let data = data as *mut dyn OptionsViewProxy;
        unsafe { Self(ManagedLazy::<dyn OptionsViewProxy>::new_raw(data, lifetime).unwrap()) }
    }

    pub fn into_inner(self) -> ManagedLazy<dyn OptionsViewProxy> {
        self.0
    }

    pub fn get<T: TryFrom<usize> + Default>(&self) -> T {
        self.0
            .read()
            .map(|data| data.get())
            .and_then(|value| T::try_from(value).ok())
            .unwrap_or_default()
    }

    pub fn set<T: TryInto<usize>>(&mut self, value: T) {
        if let Some(mut data) = self.0.write()
            && let Ok(value) = value.try_into()
        {
            data.set(value);
        }
    }
}

impl std::fmt::Debug for OptionsInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("OptionsInput")
            .field(&self.0.read().map(|data| data.get()).unwrap_or_default())
            .finish()
    }
}

impl<T: OptionsViewProxy + 'static> From<ManagedLazy<T>> for OptionsInput {
    fn from(value: ManagedLazy<T>) -> Self {
        Self::new(value)
    }
}

#[derive(PropsData, Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub enum OptionsViewMode {
    Selected,
    #[default]
    Option,
}

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct OptionsViewProps {
    #[serde(default)]
    #[serde(skip)]
    pub input: Option<OptionsInput>,
}

impl OptionsViewProps {
    pub fn get_index(&self) -> usize {
        self.input
            .as_ref()
            .map(|input| input.get::<usize>())
            .unwrap_or_default()
    }

    pub fn set_index(&mut self, value: usize) {
        if let Some(input) = self.input.as_mut() {
            input.set(value);
        }
    }
}

fn use_options_view(context: &mut WidgetContext) {
    context.life_cycle.change(|context| {
        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<ButtonNotifyMessage>()
                && msg.trigger_stop()
            {
                if msg.sender.key() == "button-selected" {
                    let mut state = context.state.read_cloned_or_default::<ContextBoxProps>();
                    state.show = !state.show;
                    let _ = context.state.write_with(state);
                } else if msg.sender.key() == "button-item" {
                    let mut state = context.state.read_cloned_or_default::<ContextBoxProps>();
                    state.show = !state.show;
                    let _ = context.state.write_with(state);
                    let params = WidgetIdMetaParams::new(msg.sender.meta());
                    if let Some(value) = params.find_value("index")
                        && let Ok(value) = value.parse::<usize>()
                        && let Ok(mut options) = context.props.read_cloned::<OptionsViewProps>()
                    {
                        options.set_index(value);
                    }
                }
            }
        }
    });
}

#[pre_hooks(use_options_view)]
pub fn options_view(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        idref,
        key,
        props,
        state,
        named_slots,
        listed_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => content);

    let state = state.read_cloned_or_default::<ContextBoxProps>();
    let active = props.read_cloned::<NavItemActive>().ok();
    let options = props.read_cloned_or_default::<OptionsViewProps>();
    let selected = listed_slots
        .get(options.get_index())
        .cloned()
        .map(|mut node| {
            node.remap_props(|props| props.with(OptionsViewMode::Selected));
            node
        })
        .unwrap_or_default();
    let content = if state.show {
        let content = match content {
            WidgetNode::Component(node) => {
                WidgetNode::Component(node.listed_slots(listed_slots.into_iter().enumerate().map(
                    |(index, mut slot)| {
                        slot.remap_props(|props| props.with(OptionsViewMode::Option));
                        make_widget!(button)
                            .key(format!("button-item?index={index}"))
                            .merge_props(slot.props().cloned().unwrap_or_default())
                            .with_props(ButtonNotifyProps(id.to_owned().into()))
                            .named_slot("content", slot)
                    },
                )))
            }
            node => node,
        };
        Some(
            make_widget!(size_box)
                .key("context")
                .merge_props(content.props().cloned().unwrap_or_default())
                .with_props(props.read_cloned_or_default::<SizeBoxProps>())
                .named_slot("content", content),
        )
    } else {
        None
    };

    make_widget!(portals_context_box)
        .key(key)
        .maybe_idref(idref.cloned())
        .with_props(props.read_cloned_or_default::<PivotBoxProps>())
        .with_props(state)
        .named_slot(
            "content",
            make_widget!(button)
                .key("button-selected")
                .maybe_with_props(active)
                .with_props(ButtonNotifyProps(id.to_owned().into()))
                .named_slot("content", selected),
        )
        .maybe_named_slot("context", content)
        .into()
}
