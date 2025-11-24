use internal::immediate_effects_box;
use raui_core::{
    DynamicManaged, DynamicManagedLazy, Lifetime, ManagedLazy, Prefab, PropsData, TypeHash,
    make_widget,
    props::{Props, PropsData},
    widget::{
        WidgetRef, component::WidgetComponent, context::WidgetContext, node::WidgetNode,
        unit::WidgetUnitNode,
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

pub fn slot_component<R>(
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
            .named_slots(widgets.into_iter().filter_map(|widget| {
                let name = widget.as_component()?.key.as_deref()?.to_owned();
                Some((name, widget))
            })),
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

pub trait ImmediateApply: Sized {
    fn before(self) -> Self {
        self
    }

    fn after(self) -> Self {
        self
    }

    fn process(self, widgets: Vec<WidgetNode>) -> Vec<WidgetNode> {
        widgets
    }
}

macro_rules! impl_tuple_immediate_apply {
    ($($id:ident),+ $(,)?) => {
        #[allow(non_snake_case)]
        impl<$($id: $crate::ImmediateApply),+> $crate::ImmediateApply for ($($id,)+) {
            fn before(self) -> Self {
                let ($($id,)+) = self;
                (
                    $(
                        $id.before(),
                    )+
                )
            }

            fn after(self) -> Self {
                let ($($id,)+) = self;
                (
                    $(
                        $id.after(),
                    )+
                )
            }

            fn process(self, mut widgets: Vec<WidgetNode>) -> Vec<WidgetNode> {
                let ($($id,)+) = self;
                $(
                    widgets = $id.process(widgets);
                )+
                widgets
            }
        }
    };
}

impl_tuple_immediate_apply!(A);
impl_tuple_immediate_apply!(A, B);
impl_tuple_immediate_apply!(A, B, C);
impl_tuple_immediate_apply!(A, B, C, D);
impl_tuple_immediate_apply!(A, B, C, D, E);
impl_tuple_immediate_apply!(A, B, C, D, E, F);
impl_tuple_immediate_apply!(A, B, C, D, E, F, G);
impl_tuple_immediate_apply!(A, B, C, D, E, F, G, H);
impl_tuple_immediate_apply!(A, B, C, D, E, F, G, H, I);
impl_tuple_immediate_apply!(A, B, C, D, E, F, G, H, I, J);
impl_tuple_immediate_apply!(A, B, C, D, E, F, G, H, I, J, K);
impl_tuple_immediate_apply!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_tuple_immediate_apply!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_tuple_immediate_apply!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_tuple_immediate_apply!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_tuple_immediate_apply!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_tuple_immediate_apply!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_tuple_immediate_apply!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_tuple_immediate_apply!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_tuple_immediate_apply!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
impl_tuple_immediate_apply!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U
);
impl_tuple_immediate_apply!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V
);
impl_tuple_immediate_apply!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, X
);
impl_tuple_immediate_apply!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, X, Y
);
impl_tuple_immediate_apply!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, X, Y, Z
);

pub struct ImKey<T: ToString>(pub T);

impl<T: ToString> ImmediateApply for ImKey<T> {
    fn process(self, mut widgets: Vec<WidgetNode>) -> Vec<WidgetNode> {
        let key = self.0.to_string();
        match widgets.len() {
            0 => {}
            1 => {
                if let WidgetNode::Component(widget) = &mut widgets[0] {
                    widget.key = Some(key);
                }
            }
            _ => {
                for (index, widget) in widgets.iter_mut().enumerate() {
                    if let WidgetNode::Component(widget) = widget {
                        widget.key = Some(format!("{key}-{index}"));
                    }
                }
            }
        }
        widgets
    }
}

pub struct ImIdRef<T: Into<WidgetRef>>(pub T);

impl<T: Into<WidgetRef>> ImmediateApply for ImIdRef<T> {
    fn process(self, mut widgets: Vec<WidgetNode>) -> Vec<WidgetNode> {
        let idref = self.0.into();
        for widget in &mut widgets {
            if let WidgetNode::Component(widget) = widget {
                widget.idref = Some(idref.clone());
            }
        }
        widgets
    }
}

pub struct ImProps<T: Into<Props>>(pub T);

impl<T: Into<Props>> ImmediateApply for ImProps<T> {
    fn process(self, mut widgets: Vec<WidgetNode>) -> Vec<WidgetNode> {
        let props = self.0.into();
        for widget in &mut widgets {
            if let Some(widget) = widget.props_mut() {
                widget.merge_from(props.clone());
            }
        }
        widgets
    }
}

pub struct ImSharedProps<T: Into<Props>>(pub T);

impl<T: Into<Props>> ImmediateApply for ImSharedProps<T> {
    fn process(self, mut widgets: Vec<WidgetNode>) -> Vec<WidgetNode> {
        let props = self.0.into();
        for widget in &mut widgets {
            if let Some(widget) = widget.shared_props_mut() {
                widget.merge_from(props.clone());
            }
        }
        widgets
    }
}

pub enum ImStackProps<T: Into<Props>> {
    Props(T),
    Done,
}

impl<T: Into<Props>> ImStackProps<T> {
    pub fn new(props: T) -> Self {
        Self::Props(props)
    }
}

impl<T: Into<Props>> ImmediateApply for ImStackProps<T> {
    fn before(self) -> Self {
        if let Self::Props(props) = self {
            let props = props.into();
            PROPS_STACK.with(|props_stack| {
                if let Some(props_stack) = props_stack.borrow_mut().as_mut() {
                    props_stack.borrow_mut().push(props.clone());
                }
            });
        }
        Self::Done
    }

    fn after(self) -> Self {
        if let Self::Done = self {
            PROPS_STACK.with(|props_stack| {
                if let Some(props_stack) = props_stack.borrow_mut().as_mut() {
                    props_stack.borrow_mut().pop();
                }
            });
        }
        self
    }
}

pub fn apply<R>(items: impl ImmediateApply, mut f: impl FnMut() -> R) -> R {
    begin();
    let items = items.before();
    let result = f();
    let items = items.after();
    let widgets = end();
    let widgets = items.process(widgets);
    extend(widgets);
    result
}

#[deprecated(note = "Use `apply` with `ImKey` instead")]
pub fn apply_key<R>(key: impl ToString, f: impl FnMut() -> R) -> R {
    apply(ImKey(key), f)
}

#[deprecated(note = "Use `apply` with `ImIdRef` instead")]
pub fn apply_idref<R>(key: impl Into<WidgetRef>, f: impl FnMut() -> R) -> R {
    apply(ImIdRef(key), f)
}

#[deprecated(note = "Use `apply` with `ImProps` instead")]
pub fn apply_props<R>(props: impl Into<Props>, f: impl FnMut() -> R) -> R {
    apply(ImProps(props), f)
}

#[deprecated(note = "Use `apply` with `ImSharedProps` instead")]
pub fn apply_shared_props<R>(props: impl Into<Props>, f: impl FnMut() -> R) -> R {
    apply(ImSharedProps(props), f)
}

#[deprecated(note = "Use `apply` with `ImStackProps` instead")]
pub fn stack_props<R>(props: impl Into<Props>, f: impl FnMut() -> R) -> R {
    apply(ImStackProps::new(props), f)
}

mod internal {
    use super::*;
    use raui_core::{unpack_named_slots, widget::unit::area::AreaBoxNode};

    pub(crate) fn immediate_effects_box(mut ctx: WidgetContext) -> WidgetNode {
        for hook in ctx.props.read_cloned_or_default::<ImmediateHooks>().hooks {
            hook(&mut ctx);
        }

        if let Ok(event) = ctx.props.read::<ImmediateOnMount>()
            && let Some(callback) = event.callback.as_ref()
        {
            let callback = callback.clone();
            ctx.life_cycle.mount(move |_| {
                callback();
            });
        }
        if let Ok(event) = ctx.props.read::<ImmediateOnChange>()
            && let Some(callback) = event.callback.as_ref()
        {
            let callback = callback.clone();
            ctx.life_cycle.change(move |_| {
                callback();
            });
        }
        if let Ok(event) = ctx.props.read::<ImmediateOnUnmount>()
            && let Some(callback) = event.callback.as_ref()
        {
            let callback = callback.clone();
            ctx.life_cycle.unmount(move |_| {
                callback();
            });
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
    use raui_core::widget::component::image_box::{ImageBoxProps, image_box};

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

    #[test]
    fn test_apply() {
        let context = ImmediateContext::default();
        ImmediateContext::activate(&context);
        begin();

        apply(
            (
                ImKey("image"),
                ImProps(ImageBoxProps::colored(Default::default())),
            ),
            || {
                component(make_widget!(image_box), ());
            },
        );

        let widgets = end();
        ImmediateContext::deactivate();

        assert_eq!(widgets.len(), 1);
        if let WidgetNode::Component(component) = &widgets[0] {
            assert_eq!(component.key.as_deref(), Some("image"));
            assert!(component.props.has::<ImageBoxProps>());
        } else {
            panic!("Expected a component node");
        }
    }
}
