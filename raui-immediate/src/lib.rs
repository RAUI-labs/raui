use internal::immediate_effects_box;
use raui_core::{
    DynamicManaged, DynamicManagedLazy, Lifetime, ManagedLazy, Prefab, PropsData, TypeHash,
    make_widget,
    props::{Props, PropsData},
    widget::{
        component::WidgetComponent, context::WidgetContext, node::WidgetNode, unit::WidgetUnitNode,
    },
};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

thread_local! {
    pub(crate) static STACK: RefCell<Vec<Vec<WidgetNode>>> = Default::default();
    pub(crate) static STATES: RefCell<Option<Rc<RefCell<ImmediateStates>>>> = Default::default();
    pub(crate) static ACCESS_POINTS: RefCell<Option<Rc<RefCell<ImmediateAccessPoints>>>> = Default::default();
    pub(crate) static PROPS_STACK: RefCell<Option<Rc<RefCell<Vec<Props>>>>> = Default::default();
}

#[derive(Default)]
pub struct ImmediateContext {
    states: Rc<RefCell<ImmediateStates>>,
    access_points: Rc<RefCell<ImmediateAccessPoints>>,
    props_stack: Rc<RefCell<Vec<Props>>>,
}

impl ImmediateContext {
    pub fn activate(context: &Self) {
        STATES.with(|states| {
            context.states.borrow_mut().reset();
            *states.borrow_mut() = Some(context.states.clone());
        });
        ACCESS_POINTS.with(|access_points| {
            *access_points.borrow_mut() = Some(context.access_points.clone());
        });
        PROPS_STACK.with(|props_stack| {
            *props_stack.borrow_mut() = Some(context.props_stack.clone());
        });
    }

    pub fn deactivate() {
        STATES.with(|states| {
            *states.borrow_mut() = None;
        });
        ACCESS_POINTS.with(|access_points| {
            if let Some(access_points) = access_points.borrow_mut().as_mut() {
                access_points.borrow_mut().reset();
            }
            *access_points.borrow_mut() = None;
        });
        PROPS_STACK.with(|props_stack| {
            if let Some(props_stack) = props_stack.borrow_mut().as_mut() {
                props_stack.borrow_mut().clear();
            }
            *props_stack.borrow_mut() = None;
        });
    }
}

#[derive(Default)]
struct ImmediateStates {
    data: Vec<DynamicManaged>,
    position: usize,
}

impl ImmediateStates {
    fn reset(&mut self) {
        self.data.truncate(self.position);
        self.position = 0;
    }

    fn alloc<T>(&mut self, mut init: impl FnMut() -> T) -> ManagedLazy<T> {
        let index = self.position;
        self.position += 1;
        if let Some(managed) = self.data.get_mut(index) {
            if managed.type_hash() != &TypeHash::of::<T>() {
                *managed = DynamicManaged::new(init()).ok().unwrap();
            }
        } else {
            self.data.push(DynamicManaged::new(init()).ok().unwrap());
        }
        self.data
            .get(index)
            .unwrap()
            .lazy()
            .into_typed()
            .ok()
            .unwrap()
    }
}

#[derive(Default)]
struct ImmediateAccessPoints {
    data: HashMap<String, DynamicManagedLazy>,
}

impl ImmediateAccessPoints {
    fn register<T>(&mut self, id: impl ToString, data: &mut T) -> Lifetime {
        let result = Lifetime::default();
        self.data
            .insert(id.to_string(), DynamicManagedLazy::new(data, result.lazy()));
        result
    }

    fn reset(&mut self) {
        self.data.clear();
    }

    fn access<T>(&self, id: &str) -> ManagedLazy<T> {
        self.data
            .get(id)
            .unwrap()
            .clone()
            .into_typed()
            .ok()
            .unwrap()
    }
}

#[derive(PropsData, Default, Clone, Serialize, Deserialize)]
#[props_data(raui_core::props::PropsData)]
pub struct ImmediateHooks {
    #[serde(default, skip)]
    hooks: Vec<fn(&mut WidgetContext)>,
}

impl ImmediateHooks {
    pub fn with(mut self, pointer: fn(&mut WidgetContext)) -> Self {
        self.hooks.push(pointer);
        self
    }
}

impl std::fmt::Debug for ImmediateHooks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(stringify!(ImmediateHooks))
            .finish_non_exhaustive()
    }
}

macro_rules! impl_lifecycle_props {
    ($($id:ident),+ $(,)?) => {
        $(
            #[derive(PropsData, Default, Clone, Serialize, Deserialize)]
            #[props_data(raui_core::props::PropsData)]
            pub struct $id {
                #[serde(default, skip)]
                callback: Option<Arc<dyn Fn() + Send + Sync>>,
            }

            impl $id {
                pub fn new(callback: impl Fn() + Send + Sync + 'static) -> Self {
                    Self {
                        callback: Some(Arc::new(callback)),
                    }
                }
            }

            impl std::fmt::Debug for $id {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.debug_struct(stringify!($id)).finish_non_exhaustive()
                }
            }
        )+
    };
}

impl_lifecycle_props! {
    ImmediateOnMount,
    ImmediateOnChange,
    ImmediateOnUnmount
}

pub fn use_state<T>(init: impl FnMut() -> T) -> ManagedLazy<T> {
    STATES.with(|states| {
        let states = states.borrow();
        let mut states = states
            .as_ref()
            .unwrap_or_else(|| panic!("You must activate context first for `use_state` to work!"))
            .borrow_mut();
        states.alloc(init)
    })
}

pub fn use_access<T>(id: &str) -> ManagedLazy<T> {
    ACCESS_POINTS.with(|access_points| {
        let access_points = access_points.borrow();
        let access_points = access_points
            .as_ref()
            .unwrap_or_else(|| panic!("You must activate context first for `use_access` to work!"))
            .borrow();
        access_points.access(id)
    })
}

pub fn use_stack_props<T: PropsData + Clone + 'static>() -> Option<T> {
    PROPS_STACK.with(|props_stack| {
        if let Some(props_stack) = props_stack.borrow().as_ref() {
            for props in props_stack.borrow().iter().rev() {
                if let Ok(props) = props.read_cloned::<T>() {
                    return Some(props);
                }
            }
        }
        None
    })
}

pub fn use_effects<R>(props: impl Into<Props>, mut f: impl FnMut() -> R) -> R {
    begin();
    let result = f();
    let node = end().pop().unwrap_or_default();
    push(
        make_widget!(immediate_effects_box)
            .merge_props(props.into())
            .named_slot("content", node),
    );
    result
}

pub fn register_access<T>(id: &str, data: &mut T) -> Lifetime {
    ACCESS_POINTS.with(|access_points| {
        let access_points = access_points.borrow();
        let mut access_points = access_points
            .as_ref()
            .unwrap_or_else(|| panic!("You must activate context first for `use_access` to work!"))
            .borrow_mut();
        access_points.register(id, data)
    })
}

pub fn begin() {
    STACK.with(|stack| stack.borrow_mut().push(Default::default()));
}

pub fn end() -> Vec<WidgetNode> {
    STACK.with(|stack| stack.borrow_mut().pop().unwrap_or_default())
}

pub fn push(widget: impl Into<WidgetNode>) {
    STACK.with(|stack| {
        if let Some(widgets) = stack.borrow_mut().last_mut() {
            widgets.push(widget.into());
        }
    });
}

pub fn extend(iter: impl IntoIterator<Item = WidgetNode>) {
    STACK.with(|stack| {
        if let Some(widgets) = stack.borrow_mut().last_mut() {
            widgets.extend(iter);
        }
    });
}

pub fn pop() -> WidgetNode {
    STACK.with(|stack| {
        stack
            .borrow_mut()
            .last_mut()
            .and_then(|widgets| widgets.pop())
            .unwrap_or_default()
    })
}

pub fn reset() {
    STACK.with(|stack| {
        stack.borrow_mut().clear();
    });
    PROPS_STACK.with(|props_stack| {
        if let Some(props_stack) = props_stack.borrow_mut().as_mut() {
            props_stack.borrow_mut().clear();
        }
    });
}

pub fn apply_key<R>(key: impl ToString, mut f: impl FnMut() -> R) -> R {
    let key = key.to_string();
    begin();
    let result = f();
    let mut widgets = end();
    match widgets.len() {
        0 => {}
        1 => {
            let mut widget = widgets.pop().unwrap();
            if let WidgetNode::Component(widget) = &mut widget {
                widget.key = Some(key);
            }
            push(widget);
        }
        _ => {
            for (index, widget) in widgets.iter_mut().enumerate() {
                if let WidgetNode::Component(widget) = widget {
                    widget.key = Some(format!("{key}-{index}"));
                }
            }
            extend(widgets);
        }
    }
    result
}

pub fn apply_props<R>(props: impl Into<Props>, mut f: impl FnMut() -> R) -> R {
    let props = props.into();
    begin();
    let result = f();
    let mut widgets = end();
    for widget in &mut widgets {
        if let Some(widget) = widget.props_mut() {
            widget.merge_from(props.clone());
        }
    }
    extend(widgets);
    result
}

pub fn apply_shared_props<R>(props: impl Into<Props>, mut f: impl FnMut() -> R) -> R {
    let props = props.into();
    begin();
    let result = f();
    let mut widgets = end();
    for widget in &mut widgets {
        if let Some(widget) = widget.shared_props_mut() {
            widget.merge_from(props.clone());
        }
    }
    extend(widgets);
    result
}

pub fn stack_props<R>(props: impl Into<Props>, mut f: impl FnMut() -> R) -> R {
    PROPS_STACK.with(|props_stack| {
        if let Some(props_stack) = props_stack.borrow_mut().as_mut() {
            props_stack.borrow_mut().push(props.into());
        }
    });
    begin();
    let result = f();
    extend(end());
    PROPS_STACK.with(|props_stack| {
        if let Some(props_stack) = props_stack.borrow_mut().as_mut() {
            props_stack.borrow_mut().pop();
        }
    });
    result
}

pub fn list_component<R>(
    widget: impl Into<WidgetComponent>,
    props: impl Into<Props>,
    mut f: impl FnMut() -> R,
) -> R {
    begin();
    let result = f();
    let widgets = end();
    push(
        widget
            .into()
            .merge_props(props.into())
            .listed_slots(widgets),
    );
    result
}

pub fn content_component<R>(
    widget: impl Into<WidgetComponent>,
    content_name: &str,
    props: impl Into<Props>,
    mut f: impl FnMut() -> R,
) -> R {
    begin();
    let result = f();
    let node = end().pop().unwrap_or_default();
    push(
        widget
            .into()
            .merge_props(props.into())
            .named_slot(content_name, node),
    );
    result
}

pub fn tuple<R>(mut f: impl FnMut() -> R) -> R {
    begin();
    let result = f();
    let widgets = end();
    push(WidgetNode::Tuple(widgets));
    result
}

pub fn component(widget: impl Into<WidgetComponent>, props: impl Into<Props>) {
    push(widget.into().merge_props(props.into()));
}

pub fn unit(widget: impl Into<WidgetUnitNode>) {
    push(widget.into());
}

pub fn make_widgets(context: &ImmediateContext, mut f: impl FnMut()) -> Vec<WidgetNode> {
    ImmediateContext::activate(context);
    begin();
    f();
    let result = end();
    ImmediateContext::deactivate();
    result
}

mod internal {
    use super::*;
    use raui_core::{unpack_named_slots, widget::unit::area::AreaBoxNode};

    pub(crate) fn immediate_effects_box(mut ctx: WidgetContext) -> WidgetNode {
        for hook in ctx.props.read_cloned_or_default::<ImmediateHooks>().hooks {
            hook(&mut ctx);
        }

        if let Ok(event) = ctx.props.read::<ImmediateOnMount>() {
            if let Some(callback) = event.callback.as_ref() {
                let callback = callback.clone();
                ctx.life_cycle.mount(move |_| {
                    callback();
                });
            }
        }
        if let Ok(event) = ctx.props.read::<ImmediateOnChange>() {
            if let Some(callback) = event.callback.as_ref() {
                let callback = callback.clone();
                ctx.life_cycle.change(move |_| {
                    callback();
                });
            }
        }
        if let Ok(event) = ctx.props.read::<ImmediateOnUnmount>() {
            if let Some(callback) = event.callback.as_ref() {
                let callback = callback.clone();
                ctx.life_cycle.unmount(move |_| {
                    callback();
                });
            }
        }

        unpack_named_slots!(ctx.named_slots => content);

        AreaBoxNode {
            id: ctx.id.to_owned(),
            slot: Box::new(content),
        }
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(frame: usize) {
        let show_slider = use_state(|| false);
        let mut show_slider = show_slider.write().unwrap();

        let show_text_field = use_state(|| false);
        let mut show_text_field = show_text_field.write().unwrap();

        if frame == 1 {
            *show_slider = true;
        } else if frame == 3 {
            *show_text_field = true;
        } else if frame == 5 {
            *show_slider = false;
        } else if frame == 7 {
            *show_text_field = false;
        } else if frame == 9 {
            *show_slider = true;
            *show_text_field = true;
        }

        println!(
            "* #{} | HOVERED: {} | CLICKED: {}",
            frame, *show_slider, *show_text_field
        );

        if *show_slider {
            slider();
        }
        if *show_text_field {
            text_field();
        }
    }

    fn slider() {
        let value = use_state(|| 0.0);
        let mut state = value.write().unwrap();

        *state += 0.1;
        println!("* SLIDER VALUE: {}", *state);
    }

    fn text_field() {
        let text = use_state(String::default);
        let mut text = text.write().unwrap();

        text.push('z');

        println!("* TEXT FIELD: {}", text.as_str());
    }

    #[test]
    fn test_use_state() {
        let context = ImmediateContext::default();
        for frame in 0..12 {
            ImmediateContext::activate(&context);
            run(frame);
            ImmediateContext::deactivate();
        }
    }
}
