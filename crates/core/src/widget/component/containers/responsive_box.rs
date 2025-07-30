use crate::{
    PropsData, Scalar, pre_hooks, unpack_named_slots,
    view_model::{ViewModelProperties, ViewModelValue},
    widget::{
        component::{ResizeListenerSignal, use_resize_listener},
        context::WidgetContext,
        node::WidgetNode,
        unit::area::AreaBoxNode,
        utils::Vec2,
    },
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub struct MediaQueryContext<'a> {
    pub widget_width: Scalar,
    pub widget_height: Scalar,
    pub view_model: Option<&'a MediaQueryViewModel>,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum MediaQueryOrientation {
    #[default]
    Portrait,
    Landscape,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum MediaQueryNumber {
    Exact(Scalar),
    Min(Scalar),
    Max(Scalar),
    Range { min: Scalar, max: Scalar },
}

impl Default for MediaQueryNumber {
    fn default() -> Self {
        Self::Exact(0.0)
    }
}

impl MediaQueryNumber {
    pub fn is_valid(&self, value: Scalar) -> bool {
        match self {
            Self::Exact(v) => (*v - value).abs() < Scalar::EPSILON,
            Self::Min(v) => *v <= value,
            Self::Max(v) => *v >= value,
            Self::Range { min, max } => *min <= value && *max >= value,
        }
    }
}

#[derive(PropsData, Debug, Default, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub enum MediaQueryExpression {
    #[default]
    Any,
    And(Vec<Self>),
    Or(Vec<Self>),
    Not(Box<Self>),
    WidgetOrientation(MediaQueryOrientation),
    WidgetAspectRatio(MediaQueryNumber),
    WidgetWidth(MediaQueryNumber),
    WidgetHeight(MediaQueryNumber),
    ScreenOrientation(MediaQueryOrientation),
    ScreenAspectRatio(MediaQueryNumber),
    ScreenWidth(MediaQueryNumber),
    ScreenHeight(MediaQueryNumber),
    HasFlag(String),
    HasNumber(String),
    Number(String, MediaQueryNumber),
}

impl MediaQueryExpression {
    pub fn is_valid(&self, context: &MediaQueryContext) -> bool {
        match self {
            Self::Any => true,
            Self::And(conditions) => conditions
                .iter()
                .all(|condition| condition.is_valid(context)),
            Self::Or(conditions) => conditions
                .iter()
                .any(|condition| condition.is_valid(context)),
            Self::Not(condition) => !condition.is_valid(context),
            Self::WidgetOrientation(orientation) => {
                let is_portrait = context.widget_height > context.widget_width;
                match orientation {
                    MediaQueryOrientation::Portrait => is_portrait,
                    MediaQueryOrientation::Landscape => !is_portrait,
                }
            }
            Self::WidgetAspectRatio(aspect_ratio) => {
                let ratio = context.widget_width / context.widget_height;
                aspect_ratio.is_valid(ratio)
            }
            Self::WidgetWidth(width) => width.is_valid(context.widget_width),
            Self::WidgetHeight(height) => height.is_valid(context.widget_height),
            Self::ScreenOrientation(orientation) => context
                .view_model
                .map(|view_model| {
                    let is_portrait = view_model.screen_size.y > view_model.screen_size.x;
                    match orientation {
                        MediaQueryOrientation::Portrait => is_portrait,
                        MediaQueryOrientation::Landscape => !is_portrait,
                    }
                })
                .unwrap_or_default(),
            Self::ScreenAspectRatio(aspect_ratio) => context
                .view_model
                .map(|view_model| {
                    let ratio = view_model.screen_size.x / view_model.screen_size.y;
                    aspect_ratio.is_valid(ratio)
                })
                .unwrap_or_default(),
            Self::ScreenWidth(width) => context
                .view_model
                .map(|view_model| width.is_valid(view_model.screen_size.x))
                .unwrap_or_default(),
            Self::ScreenHeight(height) => context
                .view_model
                .map(|view_model| height.is_valid(view_model.screen_size.y))
                .unwrap_or_default(),
            Self::HasFlag(flag) => context
                .view_model
                .map(|view_model| view_model.flags.contains(flag))
                .unwrap_or_default(),
            Self::HasNumber(name) => context
                .view_model
                .map(|view_model| view_model.numbers.contains_key(name))
                .unwrap_or_default(),
            Self::Number(name, number) => context
                .view_model
                .map(|view_model| {
                    view_model
                        .numbers
                        .get(name)
                        .map(|value| number.is_valid(*value))
                        .unwrap_or_default()
                })
                .unwrap_or_default(),
        }
    }
}

#[derive(PropsData, Debug, Default, Copy, Clone, Serialize, Deserialize)]
#[props_data(crate::props::PropsData)]
#[prefab(crate::Prefab)]
pub struct ResponsiveBoxState {
    pub size: Vec2,
}

pub fn use_responsive_box(context: &mut WidgetContext) {
    context.life_cycle.mount(|mut context| {
        if let Some(mut bindings) = context.view_models.bindings(
            MediaQueryViewModel::VIEW_MODEL,
            MediaQueryViewModel::NOTIFIER,
        ) {
            bindings.bind(context.id.to_owned());
        }
    });

    context.life_cycle.unmount(|mut context| {
        if let Some(mut bindings) = context.view_models.bindings(
            MediaQueryViewModel::VIEW_MODEL,
            MediaQueryViewModel::NOTIFIER,
        ) {
            bindings.unbind(context.id);
        }
    });

    context.life_cycle.change(|context| {
        for msg in context.messenger.messages {
            if let Some(ResizeListenerSignal::Change(size)) = msg.as_any().downcast_ref() {
                let _ = context.state.write(ResponsiveBoxState { size: *size });
            }
        }
    });
}

#[pre_hooks(use_resize_listener, use_responsive_box)]
pub fn responsive_box(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        state,
        mut listed_slots,
        view_models,
        ..
    } = context;

    let state = state.read_cloned_or_default::<ResponsiveBoxState>();
    let view_model = view_models
        .view_model(MediaQueryViewModel::VIEW_MODEL)
        .and_then(|vm| vm.read::<MediaQueryViewModel>());
    let ctx = MediaQueryContext {
        widget_width: state.size.x,
        widget_height: state.size.y,
        view_model: view_model.as_deref(),
    };
    let item = if let Some(index) = listed_slots.iter().position(|slot| {
        slot.props()
            .map(|props| {
                props
                    .read::<MediaQueryExpression>()
                    .ok()
                    .map(|query| query.is_valid(&ctx))
                    .unwrap_or(true)
            })
            .unwrap_or_default()
    }) {
        listed_slots.remove(index)
    } else {
        Default::default()
    };

    AreaBoxNode {
        id: id.to_owned(),
        slot: Box::new(item),
    }
    .into()
}

#[pre_hooks(use_resize_listener, use_responsive_box)]
pub fn responsive_props_box(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        state,
        listed_slots,
        named_slots,
        view_models,
        ..
    } = context;
    unpack_named_slots!(named_slots => content);

    let state = state.read_cloned_or_default::<ResponsiveBoxState>();
    let view_model = view_models
        .view_model(MediaQueryViewModel::VIEW_MODEL)
        .and_then(|vm| vm.read::<MediaQueryViewModel>());
    let ctx = MediaQueryContext {
        widget_width: state.size.x,
        widget_height: state.size.y,
        view_model: view_model.as_deref(),
    };

    let props = listed_slots
        .iter()
        .find_map(|slot| {
            slot.props().and_then(|props| {
                props
                    .read::<MediaQueryExpression>()
                    .ok()
                    .map(|query| query.is_valid(&ctx))
                    .unwrap_or(true)
                    .then_some(props.clone())
            })
        })
        .unwrap_or_default();

    if let Some(p) = content.props_mut() {
        p.merge_from(props.without::<MediaQueryExpression>());
    }

    AreaBoxNode {
        id: id.to_owned(),
        slot: Box::new(content),
    }
    .into()
}

#[derive(Debug)]
pub struct MediaQueryViewModel {
    pub screen_size: ViewModelValue<Vec2>,
    pub flags: ViewModelValue<HashSet<String>>,
    pub numbers: ViewModelValue<HashMap<String, Scalar>>,
}

impl MediaQueryViewModel {
    pub const VIEW_MODEL: &str = "MediaQueryViewModel";
    pub const NOTIFIER: &str = "";

    pub fn new(properties: &mut ViewModelProperties) -> Self {
        let notifier = properties.notifier(Self::NOTIFIER);
        Self {
            screen_size: ViewModelValue::new(Default::default(), notifier.clone()),
            flags: ViewModelValue::new(Default::default(), notifier.clone()),
            numbers: ViewModelValue::new(Default::default(), notifier.clone()),
        }
    }
}
