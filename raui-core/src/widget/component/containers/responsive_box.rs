use crate::{
    PropsData, Scalar, pre_hooks,
    widget::{
        component::{ResizeListenerSignal, use_resize_listener},
        context::WidgetContext,
        node::WidgetNode,
        unit::area::AreaBoxNode,
        utils::Vec2,
    },
};
use serde::{Deserialize, Serialize};

pub struct MediaQueryContext {
    pub widget_width: Scalar,
    pub widget_height: Scalar,
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
    context.life_cycle.change(|context| {
        for msg in context.messenger.messages {
            if let Some(ResizeListenerSignal::Change(size)) = msg.as_any().downcast_ref() {
                let _ = context.state.write(ResponsiveBoxState { size: *size });
            }
        }
    });
}

#[pre_hooks(use_responsive_box, use_resize_listener)]
pub fn responsive_box(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        id,
        state,
        mut listed_slots,
        ..
    } = context;

    let state = state.read_cloned_or_default::<ResponsiveBoxState>();
    let ctx = MediaQueryContext {
        widget_width: state.size.x,
        widget_height: state.size.y,
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
