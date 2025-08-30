use raui_core::{
    Lifetime, LifetimeLazy, Managed, ValueReadAccess, ValueWriteAccess,
    application::ChangeNotifier,
    widget::{FnWidget, WidgetRef, component::WidgetComponent, context::*, node::WidgetNode},
};
use std::{
    any::Any,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

#[allow(unused_variables)]
pub trait ViewState: Any + Send + Sync {
    fn on_mount(&mut self, context: WidgetMountOrChangeContext) {}
    fn on_unmount(&mut self, context: WidgetUnmountContext) {}
    fn on_change(&mut self, context: WidgetMountOrChangeContext) {}
    fn on_render(&self, context: WidgetContext) -> WidgetNode;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct View<T: ViewState> {
    inner: Box<dyn ViewState>,
    lifetime: Box<Lifetime>,
    _phantom: PhantomData<fn() -> T>,
}

impl<T: ViewState + Default> Default for View<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: ViewState> View<T> {
    pub fn new(state: T) -> Self
    where
        T: 'static,
    {
        Self {
            inner: Box::new(state),
            lifetime: Default::default(),
            _phantom: Default::default(),
        }
    }

    pub fn into_inner(self) -> Box<dyn ViewState> {
        self.inner
    }

    pub fn as_dyn(&'_ self) -> Option<ValueReadAccess<'_, dyn ViewState>> {
        self.lifetime.read(&*self.inner)
    }

    pub fn as_dyn_mut(&'_ mut self) -> Option<ValueWriteAccess<'_, dyn ViewState>> {
        self.lifetime.write(&mut *self.inner)
    }

    pub fn read(&'_ self) -> Option<ValueReadAccess<'_, T>> {
        self.lifetime.read(self.inner.as_any().downcast_ref::<T>()?)
    }

    pub fn write(&'_ mut self) -> Option<ValueWriteAccess<'_, T>> {
        self.lifetime
            .write(self.inner.as_any_mut().downcast_mut::<T>()?)
    }

    pub fn lazy(&self) -> LazyView<T> {
        unsafe {
            let ptr = self.inner.as_any().downcast_ref::<T>().unwrap() as *const T as *mut T;
            LazyView {
                inner: NonNull::new_unchecked(ptr),
                lifetime: self.lifetime.lazy(),
            }
        }
    }

    pub fn component(&self) -> WidgetComponent {
        WidgetComponent::new(self.widget(), std::any::type_name::<Self>())
    }

    pub fn widget(&self) -> FnWidget {
        let this = self.lazy();
        FnWidget::closure(move |context| {
            let this_mount = this.clone();
            let this_unmount = this.clone();
            let this_change = this.clone();
            context.life_cycle.mount(move |context| {
                if let Some(mut this) = this_mount.write() {
                    this.on_mount(context);
                }
            });
            context.life_cycle.unmount(move |context| {
                if let Some(mut this) = this_unmount.write() {
                    this.on_unmount(context);
                }
            });
            context.life_cycle.change(move |context| {
                if let Some(mut this) = this_change.write() {
                    this.on_change(context);
                }
            });
            this.write()
                .map(|this| this.on_render(context))
                .unwrap_or_default()
        })
    }
}

pub struct LazyView<T: ViewState> {
    inner: NonNull<T>,
    lifetime: LifetimeLazy,
}

unsafe impl<T: ViewState> Send for LazyView<T> {}
unsafe impl<T: ViewState> Sync for LazyView<T> {}

impl<T: ViewState> Clone for LazyView<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner,
            lifetime: self.lifetime.clone(),
        }
    }
}

impl<T: ViewState> LazyView<T> {
    pub fn as_dyn(&'_ self) -> Option<ValueReadAccess<'_, dyn ViewState>> {
        unsafe { self.lifetime.read(self.inner.as_ref()) }
    }

    pub fn as_dyn_mut(&'_ self) -> Option<ValueWriteAccess<'_, dyn ViewState>> {
        unsafe { self.lifetime.write(self.inner.as_ptr().as_mut()?) }
    }

    pub fn read(&'_ self) -> Option<ValueReadAccess<'_, T>> {
        unsafe { self.lifetime.read(self.inner.as_ref()) }
    }

    pub fn write(&'_ self) -> Option<ValueWriteAccess<'_, T>> {
        unsafe { self.lifetime.write(self.inner.as_ptr().as_mut()?) }
    }
}

pub struct ViewValue<T> {
    inner: T,
    notifier: Option<ChangeNotifier>,
    id: WidgetRef,
}

impl<T> ViewValue<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: value,
            notifier: None,
            id: Default::default(),
        }
    }

    pub fn with_notifier(mut self, notifier: ChangeNotifier) -> Self {
        self.bind_notifier(notifier);
        self
    }

    pub fn bind_notifier(&mut self, notifier: ChangeNotifier) {
        if self.notifier.is_none()
            && let Some(id) = self.id.read()
        {
            notifier.notify(id);
        }
        self.notifier = Some(notifier);
    }

    pub fn unbind_notifier(&mut self) {
        self.notifier = None;
    }

    pub fn bound_notifier(&self) -> Option<&ChangeNotifier> {
        self.notifier.as_ref()
    }

    pub fn widget_ref(&self) -> WidgetRef {
        self.id.clone()
    }
}

impl<T> Deref for ViewValue<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for ViewValue<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if let Some(notifier) = self.notifier.as_ref()
            && let Some(id) = self.id.read()
        {
            notifier.notify(id);
        }
        &mut self.inner
    }
}

pub struct SharedView<T: ViewState> {
    inner: Managed<Option<View<T>>>,
}

impl<T: ViewState> Default for SharedView<T> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<T: ViewState> SharedView<T> {
    pub fn new(view: View<T>) -> Self {
        Self {
            inner: Managed::new(Some(view)),
        }
    }

    pub fn replace(&mut self, view: View<T>) {
        if let Some(mut inner) = self.inner.write() {
            *inner = Some(view);
        }
    }

    pub fn clear(&mut self) {
        if let Some(mut inner) = self.inner.write() {
            *inner = None;
        }
    }

    pub fn read(&'_ self) -> Option<ValueReadAccess<'_, View<T>>> {
        self.inner.read()?.remap(|inner| inner.as_ref()).ok()
    }

    pub fn write(&'_ mut self) -> Option<ValueWriteAccess<'_, View<T>>> {
        self.inner.write()?.remap(|inner| inner.as_mut()).ok()
    }
}

impl<T: ViewState> ViewState for SharedView<T> {
    fn on_render(&self, context: WidgetContext) -> WidgetNode {
        self.inner
            .read()
            .and_then(|inner| {
                inner
                    .as_ref()
                    .map(|inner| inner.component().key(context.key).into())
            })
            .unwrap_or_default()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
