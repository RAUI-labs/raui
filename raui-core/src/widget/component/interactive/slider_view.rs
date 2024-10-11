use crate::{
    pre_hooks, unpack_named_slots,
    view_model::ViewModelValue,
    widget::{
        component::interactive::{
            button::{use_button, ButtonNotifyMessage, ButtonNotifyProps, ButtonProps},
            navigation::{
                use_nav_item, use_nav_tracking_self, NavSignal, NavTrackingNotifyMessage,
                NavTrackingNotifyProps,
            },
        },
        context::WidgetContext,
        node::WidgetNode,
        unit::area::AreaBoxNode,
    },
    PropsData, Scalar,
};
use intuicio_data::managed::ManagedLazy;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

fn is_zero(value: &Scalar) -> bool {
    value.abs() < 1.0e-6
}

pub trait SliderViewProxy: Send + Sync {
    fn get(&self) -> Scalar;
    fn set(&mut self, value: Scalar);
}

macro_rules! impl_proxy {
    ($type:ty) => {
        impl SliderViewProxy for $type {
            fn get(&self) -> Scalar {
                *self as _
            }

            fn set(&mut self, value: Scalar) {
                *self = value as _;
            }
        }
    };
    (@round $type:ty) => {
        impl SliderViewProxy for $type {
            fn get(&self) -> Scalar {
                *self as _
            }

            fn set(&mut self, value: Scalar) {
                *self = value.round() as _;
            }
        }
    };
}

impl_proxy!(@round u8);
impl_proxy!(@round u16);
impl_proxy!(@round u32);
impl_proxy!(@round u64);
impl_proxy!(@round u128);
impl_proxy!(@round usize);
impl_proxy!(@round i8);
impl_proxy!(@round i16);
impl_proxy!(@round i32);
impl_proxy!(@round i64);
impl_proxy!(@round i128);
impl_proxy!(@round isize);
impl_proxy!(f32);
impl_proxy!(f64);

impl<T> SliderViewProxy for ViewModelValue<T>
where
    T: SliderViewProxy,
{
    fn get(&self) -> Scalar {
        self.deref().get()
    }

    fn set(&mut self, value: Scalar) {
        self.deref_mut().set(value);
    }
}

#[derive(Clone)]
pub struct SliderInput(ManagedLazy<dyn SliderViewProxy>);

impl SliderInput {
    pub fn new(data: ManagedLazy<impl SliderViewProxy + 'static>) -> Self {
        let (lifetime, data) = data.into_inner();
        let data = data as *mut dyn SliderViewProxy;
        unsafe { Self(ManagedLazy::<dyn SliderViewProxy>::new_raw(data, lifetime).unwrap()) }
    }

    pub fn into_inner(self) -> ManagedLazy<dyn SliderViewProxy> {
        self.0
    }

    pub fn get<T: TryFrom<Scalar> + Default>(&self) -> T {
        self.0
            .read()
            .map(|data| data.get())
            .and_then(|value| T::try_from(value).ok())
            .unwrap_or_default()
    }

    pub fn set<T: TryInto<Scalar>>(&mut self, value: T) {
        if let Some(mut data) = self.0.write() {
            if let Ok(value) = value.try_into() {
                data.set(value);
            }
        }
    }
}

impl std::fmt::Debug for SliderInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SliderInput")
            .field(&self.0.read().map(|data| data.get()).unwrap_or_default())
            .finish()
    }
}

impl<T: SliderViewProxy + 'static> From<ManagedLazy<T>> for SliderInput {
    fn from(value: ManagedLazy<T>) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SliderViewDirection {
    #[default]
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct SliderViewProps {
    #[serde(default)]
    #[serde(skip)]
    pub input: Option<SliderInput>,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero")]
    pub from: Scalar,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero")]
    pub to: Scalar,
    #[serde(default)]
    pub direction: SliderViewDirection,
}

impl SliderViewProps {
    pub fn get_value(&self) -> Scalar {
        self.input
            .as_ref()
            .map(|input| input.get::<Scalar>())
            .unwrap_or_default()
    }

    pub fn set_value(&mut self, value: Scalar) {
        if let Some(input) = self.input.as_mut() {
            input.set(value);
        }
    }

    pub fn get_percentage(&self) -> Scalar {
        (self.get_value() - self.from) / (self.to - self.from)
    }

    pub fn set_percentage(&mut self, value: Scalar) {
        self.set_value(value * (self.to - self.from) + self.from)
    }
}

#[pre_hooks(use_button, use_nav_tracking_self)]
pub fn use_slider_view(context: &mut WidgetContext) {
    context
        .props
        .write(ButtonNotifyProps(context.id.to_owned().into()));
    context
        .props
        .write(NavTrackingNotifyProps(context.id.to_owned().into()));

    context.life_cycle.change(|context| {
        for msg in context.messenger.messages {
            if let Some(msg) = msg.as_any().downcast_ref::<ButtonNotifyMessage>() {
                if msg.trigger_start() {
                    context.signals.write(NavSignal::Lock);
                }
                if msg.trigger_stop() {
                    context.signals.write(NavSignal::Unlock);
                }
            } else if let Some(msg) = msg.as_any().downcast_ref::<NavTrackingNotifyMessage>() {
                let button = context.state.read_cloned_or_default::<ButtonProps>();
                if button.selected && button.trigger {
                    let mut props = context.props.read_cloned_or_default::<SliderViewProps>();
                    let value = match props.direction {
                        SliderViewDirection::LeftToRight => msg.state.0.x,
                        SliderViewDirection::RightToLeft => 1.0 - msg.state.0.x,
                        SliderViewDirection::TopToBottom => msg.state.0.y,
                        SliderViewDirection::BottomToTop => 1.0 - msg.state.0.y,
                    }
                    .clamp(0.0, 1.0);
                    let value = value * (props.to - props.from) + props.from;
                    if let Some(input) = props.input.as_mut() {
                        input.set(value);
                    }
                }
            }
        }
    });
}

#[pre_hooks(use_nav_item, use_slider_view)]
pub fn slider_view(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        props,
        state,
        named_slots,
        ..
    } = context;
    unpack_named_slots!(named_slots => content);

    if let Some(p) = content.props_mut() {
        p.write(state.read_cloned_or_default::<ButtonProps>());
        p.write(props.read_cloned_or_default::<SliderViewProps>());
    }

    AreaBoxNode {
        id: id.to_owned(),
        slot: Box::new(content),
    }
    .into()
}
