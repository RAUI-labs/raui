use crate::widget::{WidgetId, WidgetIdCommon};
use intuicio_data::{
    lifetime::{ValueReadAccess, ValueWriteAccess},
    managed::DynamicManaged,
    managed::{Managed, ManagedLazy, ManagedRef, ManagedRefMut},
    type_hash::TypeHash,
};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    ops::{Deref, DerefMut},
};

pub struct ViewModelBindings {
    widgets: HashSet<WidgetId>,
    common_root: WidgetIdCommon,
    notify: bool,
}

impl Default for ViewModelBindings {
    fn default() -> Self {
        Self {
            widgets: Default::default(),
            common_root: Default::default(),
            notify: true,
        }
    }
}

impl ViewModelBindings {
    pub fn bind(&mut self, id: WidgetId) {
        self.widgets.insert(id);
        self.rebuild_common_root();
    }

    pub fn unbind(&mut self, id: &WidgetId) {
        self.widgets.remove(id);
        self.rebuild_common_root();
    }

    pub fn clear(&mut self) {
        self.widgets.clear();
        self.common_root = Default::default();
    }

    pub fn is_empty(&self) -> bool {
        self.widgets.is_empty()
    }

    pub fn is_bound(&self, id: &WidgetId) -> bool {
        self.widgets.contains(id)
    }

    pub fn widgets(&self) -> impl Iterator<Item = &WidgetId> {
        self.widgets.iter()
    }

    pub fn common_root(&self) -> &WidgetIdCommon {
        &self.common_root
    }

    pub fn notify(&mut self) {
        self.notify = true;
    }

    pub fn is_notified(&self) -> bool {
        self.notify
    }

    pub fn consume_notification(&mut self) -> bool {
        !self.widgets.is_empty() && std::mem::take(&mut self.notify)
    }

    fn rebuild_common_root(&mut self) {
        self.common_root = WidgetIdCommon::from_iter(self.widgets.iter());
    }
}

#[derive(Default)]
pub struct ViewModelProperties {
    inner: HashMap<String, Managed<ViewModelBindings>>,
}

impl ViewModelProperties {
    pub fn unbind_all(&mut self, id: &WidgetId) {
        for bindings in self.inner.values_mut() {
            if let Some(mut bindings) = bindings.write() {
                bindings.unbind(id);
            }
        }
    }

    pub fn remove(&mut self, id: &str) {
        self.inner.remove(id);
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn has(&self, id: &str) -> bool {
        self.inner.contains_key(id)
    }

    pub fn remove_empty_bindings(&mut self) {
        let to_remove = self
            .inner
            .iter()
            .filter_map(|(key, bindings)| {
                if let Some(bindings) = bindings.read() {
                    if bindings.is_empty() {
                        return Some(key.to_owned());
                    }
                }
                None
            })
            .collect::<Vec<_>>();
        for key in to_remove {
            self.inner.remove(&key);
        }
    }

    pub fn bindings(&mut self, id: impl ToString) -> Option<ValueWriteAccess<ViewModelBindings>> {
        self.inner.entry(id.to_string()).or_default().write()
    }

    pub fn notifier(&mut self, id: impl ToString) -> ViewModelNotifier {
        ViewModelNotifier {
            inner: self.inner.entry(id.to_string()).or_default().lazy(),
        }
    }

    pub fn consume_notification(&mut self) -> bool {
        self.inner.values_mut().any(|bindings| {
            bindings
                .write()
                .map(|mut bindings| bindings.consume_notification())
                .unwrap_or_default()
        })
    }

    pub fn consume_notified_common_root(&mut self) -> WidgetIdCommon {
        let mut result = WidgetIdCommon::default();
        for bindings in self.inner.values_mut() {
            if let Some(mut bindings) = bindings.write() {
                if bindings.consume_notification() {
                    let root = bindings.common_root();
                    result.include_other(root);
                }
            }
        }
        result
    }
}

pub struct ViewModelNotifier {
    inner: ManagedLazy<ViewModelBindings>,
}

impl ViewModelNotifier {
    pub fn notify(&mut self) -> bool {
        if let Some(mut bindings) = self.inner.write() {
            bindings.notify();
            true
        } else {
            false
        }
    }
}

pub struct ViewModel {
    object: DynamicManaged,
    pub properties: ViewModelProperties,
}

impl ViewModel {
    pub fn new<T: 'static>(object: T, properties: ViewModelProperties) -> Self {
        Self {
            object: DynamicManaged::new(object),
            properties,
        }
    }

    pub fn new_object<T: 'static>(object: T) -> Self {
        Self::new(object, Default::default())
    }

    pub fn produce<T: 'static>(producer: impl FnOnce(&mut ViewModelProperties) -> T) -> Self {
        let mut properties = Default::default();
        let object = DynamicManaged::new(producer(&mut properties));
        Self { object, properties }
    }

    pub fn borrow<T: 'static>(&self) -> Option<ManagedRef<T>> {
        self.object
            .borrow()
            .and_then(|object| object.into_typed::<T>().ok())
    }

    pub fn borrow_mut<T: 'static>(&mut self) -> Option<ManagedRefMut<T>> {
        self.object
            .borrow_mut()
            .and_then(|object| object.into_typed::<T>().ok())
    }

    pub fn lazy<T: 'static>(&mut self) -> Option<ManagedLazy<T>> {
        self.object.lazy().into_typed::<T>().ok()
    }

    pub fn read<T: 'static>(&self) -> Option<ValueReadAccess<T>> {
        self.object.read::<T>()
    }

    pub fn write<T: 'static>(&mut self) -> Option<ValueWriteAccess<T>> {
        self.object.write::<T>()
    }

    pub fn write_notified<T: 'static>(&mut self) -> Option<ViewModelObject<T>> {
        if let Some(access) = self.object.write::<T>() {
            Some(ViewModelObject {
                access,
                notifier: self.properties.notifier(""),
            })
        } else {
            None
        }
    }
}

#[derive(Default)]
pub struct ViewModelCollection {
    inner: HashMap<String, ViewModel>,
}

impl ViewModelCollection {
    pub fn unbind_all(&mut self, id: &WidgetId) {
        for view_model in self.inner.values_mut() {
            view_model.properties.unbind_all(id);
        }
    }

    pub fn remove_empty_bindings(&mut self) {
        for view_model in self.inner.values_mut() {
            view_model.properties.remove_empty_bindings();
        }
    }

    pub fn consume_notification(&mut self) -> bool {
        self.inner
            .values_mut()
            .any(|view_model| view_model.properties.consume_notification())
    }

    pub fn consume_notified_common_root(&mut self) -> WidgetIdCommon {
        let mut result = WidgetIdCommon::default();
        for view_model in self.inner.values_mut() {
            result.include_other(&view_model.properties.consume_notified_common_root());
        }
        result
    }
}

impl Deref for ViewModelCollection {
    type Target = HashMap<String, ViewModel>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ViewModelCollection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct ViewModelCollectionView<'a> {
    inner: &'a mut ViewModelCollection,
    defaults: HashMap<TypeHash, Cow<'static, str>>,
}

impl<'a> ViewModelCollectionView<'a> {
    pub fn new(
        inner: &'a mut ViewModelCollection,
        defaults: HashMap<TypeHash, Cow<'static, str>>,
    ) -> Self {
        Self { inner, defaults }
    }

    pub fn into_defaults(self) -> HashMap<TypeHash, Cow<'static, str>> {
        self.defaults
    }

    pub fn collection(&'a mut self) -> &'a mut ViewModelCollection {
        self.inner
    }

    pub fn bindings(
        &mut self,
        view_model: &str,
        property: impl ToString,
    ) -> Option<ValueWriteAccess<ViewModelBindings>> {
        self.inner
            .get_mut(view_model)?
            .properties
            .bindings(property)
    }

    pub fn default_bindings<T: 'static>(
        &mut self,
        property: impl ToString,
    ) -> Option<ValueWriteAccess<ViewModelBindings>> {
        let view_model = self.defaults.get(&TypeHash::of::<T>())?;
        self.inner
            .get_mut(view_model.as_ref())?
            .properties
            .bindings(property)
    }

    pub fn view_model<T: 'static>(&self, name: &str) -> Option<ValueReadAccess<T>> {
        self.inner.get(name)?.read::<T>()
    }

    pub fn default_view_model<T: 'static>(&self) -> Option<ValueReadAccess<T>> {
        let name = self.defaults.get(&TypeHash::of::<T>())?;
        self.inner.get(name.as_ref())?.read::<T>()
    }

    pub fn view_model_mut<T: 'static>(&mut self, name: &str) -> Option<ValueWriteAccess<T>> {
        self.inner.get_mut(name)?.write::<T>()
    }

    pub fn default_view_model_mut<T: 'static>(&mut self) -> Option<ValueWriteAccess<T>> {
        let name = self.defaults.get(&TypeHash::of::<T>())?;
        self.inner.get_mut(name.as_ref())?.write::<T>()
    }

    pub fn set_default<T: 'static>(&mut self, name: impl Into<Cow<'static, str>>) {
        let name = name.into();
        if let Some(view_model) = self.inner.get(name.as_ref()) {
            let type_hash = TypeHash::of::<T>();
            if view_model.object.type_hash() == &type_hash {
                self.defaults.insert(type_hash, name);
            }
        }
    }

    pub fn unset_default<T: 'static>(&mut self) {
        self.defaults.remove(&TypeHash::of::<T>());
    }
}

pub struct ViewModelObject<'a, T> {
    access: ValueWriteAccess<'a, T>,
    notifier: ViewModelNotifier,
}

impl<'a, T> ViewModelObject<'a, T> {
    pub fn set_unique_notify(&mut self, value: T)
    where
        T: PartialEq,
    {
        if *self.access != value {
            *self.access = value;
            self.notifier.notify();
        }
    }
}

impl<'a, T> Deref for ViewModelObject<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.access
    }
}

impl<'a, T> DerefMut for ViewModelObject<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.notifier.notify();
        &mut self.access
    }
}

pub struct ViewModelValue<T> {
    value: T,
    notifier: ViewModelNotifier,
}

impl<T> ViewModelValue<T> {
    pub fn new(value: T, notifier: ViewModelNotifier) -> Self {
        Self { value, notifier }
    }

    pub fn set_unique_notify(&mut self, value: T)
    where
        T: PartialEq,
    {
        if self.value != value {
            self.value = value;
            self.notifier.notify();
        }
    }
}

impl<T> Deref for ViewModelValue<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for ViewModelValue<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.notifier.notify();
        &mut self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    const FOO_VIEW_MODEL: &str = "foo";
    const COUNTER_PROPERTY: &str = "counter";
    const FLAG_PROPERTY: &str = "flag";

    // view-model data type
    struct Foo {
        // can hold view-model value wrapper that implicitly notifies on mutation.
        counter: ViewModelValue<usize>,
        // or can hold raw notifiers to explicitly notify.
        flag: bool,
        flag_notifier: ViewModelNotifier,
    }

    impl Foo {
        fn toggle(&mut self) {
            self.flag = !self.flag;
            self.flag_notifier.notify();
        }
    }

    #[test]
    fn test_view_model() {
        let a = WidgetId::from_str("a:root/a").unwrap();
        let b = WidgetId::from_str("b:root/b").unwrap();
        let mut collection = ViewModelCollection::default();

        // create new view-model and add it to collection.
        // `produce` method allows to setup notifiers as we construct view-model.
        let mut view_model = ViewModel::produce(|properties| Foo {
            counter: ViewModelValue::new(0, properties.notifier(COUNTER_PROPERTY)),
            flag: false,
            flag_notifier: properties.notifier(FLAG_PROPERTY),
        });
        // handle to view-model data we can use to share around.
        // it stays alive as long as its view-model object.
        let handle = view_model.lazy::<Foo>().unwrap();
        collection.insert(FOO_VIEW_MODEL.to_owned(), view_model);

        // unbound properties won't trigger notification until we bind widgets to them.
        assert_eq!(collection.consume_notified_common_root().is_valid(), false);
        handle.write().unwrap().toggle();
        assert_eq!(collection.consume_notified_common_root().is_valid(), false);
        assert!(collection
            .get_mut(FOO_VIEW_MODEL)
            .unwrap()
            .properties
            .bindings(COUNTER_PROPERTY)
            .unwrap()
            .is_notified());
        assert!(collection
            .get_mut(FOO_VIEW_MODEL)
            .unwrap()
            .properties
            .bindings(FLAG_PROPERTY)
            .unwrap()
            .is_notified());

        // bind widget to properties.
        // whenever property gets notified, its widgets will rebuild.
        collection
            .get_mut(FOO_VIEW_MODEL)
            .unwrap()
            .properties
            .bindings(COUNTER_PROPERTY)
            .unwrap()
            .bind(a);
        collection
            .get_mut(FOO_VIEW_MODEL)
            .unwrap()
            .properties
            .bindings(FLAG_PROPERTY)
            .unwrap()
            .bind(b);

        // once we bind properties, notification will be triggered.
        assert_eq!(
            collection.consume_notified_common_root().path(),
            Some("root")
        );

        // automatically notify on view-model value mutation.
        *handle.write().unwrap().counter += 1;
        assert_eq!(
            collection.consume_notified_common_root().path(),
            Some("root/a"),
        );

        // proxy notify via view-model method call.
        handle.write().unwrap().toggle();
        assert_eq!(
            collection.consume_notified_common_root().path(),
            Some("root/b"),
        );

        // rebuilding widgets tree will occur always from common root of notified widgets.
        *handle.write().unwrap().counter += 1;
        handle.write().unwrap().toggle();
        assert_eq!(
            collection.consume_notified_common_root().path(),
            Some("root"),
        );
    }
}
