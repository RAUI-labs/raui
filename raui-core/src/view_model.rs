use crate::widget::{WidgetId, WidgetIdCommon};
use intuicio_data::{
    lifetime::{ValueReadAccess, ValueWriteAccess},
    managed::DynamicManaged,
    managed::{Managed, ManagedLazy, ManagedRef, ManagedRefMut},
};
use std::{
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

#[derive(Clone)]
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
            object: DynamicManaged::new(object).ok().unwrap(),
            properties,
        }
    }

    pub fn new_object<T: 'static>(object: T) -> Self {
        Self::new(object, Default::default())
    }

    pub fn produce<T: 'static>(producer: impl FnOnce(&mut ViewModelProperties) -> T) -> Self {
        let mut properties = Default::default();
        let object = DynamicManaged::new(producer(&mut properties)).ok().unwrap();
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

    pub fn lazy<T: 'static>(&self) -> Option<ManagedLazy<T>> {
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
    named: HashMap<String, ViewModel>,
    widgets: HashMap<WidgetId, HashMap<String, ViewModel>>,
}

impl ViewModelCollection {
    pub fn unbind_all(&mut self, id: &WidgetId) {
        for view_model in self.named.values_mut() {
            view_model.properties.unbind_all(id);
        }
        for view_model in self.widgets.values_mut() {
            for view_model in view_model.values_mut() {
                view_model.properties.unbind_all(id);
            }
        }
    }

    pub fn remove_empty_bindings(&mut self) {
        for view_model in self.named.values_mut() {
            view_model.properties.remove_empty_bindings();
        }
        for view_model in self.widgets.values_mut() {
            for view_model in view_model.values_mut() {
                view_model.properties.remove_empty_bindings();
            }
        }
    }

    pub fn consume_notification(&mut self) -> bool {
        let mut result = false;
        for view_model in self.named.values_mut() {
            result = result || view_model.properties.consume_notification();
        }
        for view_model in self.widgets.values_mut() {
            for view_model in view_model.values_mut() {
                result = result || view_model.properties.consume_notification();
            }
        }
        result
    }

    pub fn consume_notified_common_root(&mut self) -> WidgetIdCommon {
        let mut result = WidgetIdCommon::default();
        for view_model in self.named.values_mut() {
            result.include_other(&view_model.properties.consume_notified_common_root());
        }
        for view_model in self.widgets.values_mut() {
            for view_model in view_model.values_mut() {
                result.include_other(&view_model.properties.consume_notified_common_root());
            }
        }
        result
    }

    pub fn remove_widget_view_models(&mut self, id: &WidgetId) {
        self.widgets.remove(id);
    }
}

impl Deref for ViewModelCollection {
    type Target = HashMap<String, ViewModel>;

    fn deref(&self) -> &Self::Target {
        &self.named
    }
}

impl DerefMut for ViewModelCollection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.named
    }
}

pub struct ViewModelCollectionView<'a> {
    id: &'a WidgetId,
    collection: &'a mut ViewModelCollection,
}

impl<'a> ViewModelCollectionView<'a> {
    pub fn new(id: &'a WidgetId, collection: &'a mut ViewModelCollection) -> Self {
        Self { id, collection }
    }

    pub fn id(&self) -> &WidgetId {
        self.id
    }

    pub fn collection(&'a self) -> &'a ViewModelCollection {
        self.collection
    }

    pub fn collection_mut(&'a mut self) -> &'a mut ViewModelCollection {
        self.collection
    }

    pub fn bindings(
        &mut self,
        view_model: &str,
        property: impl ToString,
    ) -> Option<ValueWriteAccess<ViewModelBindings>> {
        self.collection
            .get_mut(view_model)?
            .properties
            .bindings(property)
    }

    pub fn view_model(&self, name: &str) -> Option<&ViewModel> {
        self.collection.get(name)
    }

    pub fn view_model_mut(&mut self, name: &str) -> Option<&mut ViewModel> {
        self.collection.get_mut(name)
    }

    pub fn widget_register(&mut self, name: impl ToString, view_model: ViewModel) {
        self.collection
            .widgets
            .entry(self.id.to_owned())
            .or_default()
            .insert(name.to_string(), view_model);
    }

    pub fn widget_unregister(&mut self, name: &str) -> Option<ViewModel> {
        let view_models = self.collection.widgets.get_mut(self.id)?;
        let result = view_models.remove(name)?;
        if view_models.is_empty() {
            self.collection.widgets.remove(self.id);
        }
        Some(result)
    }

    pub fn widget_bindings(
        &mut self,
        view_model: &str,
        property: impl ToString,
    ) -> Option<ValueWriteAccess<ViewModelBindings>> {
        self.collection
            .widgets
            .get_mut(self.id)?
            .get_mut(view_model)?
            .properties
            .bindings(property)
    }

    pub fn widget_view_model(&self, name: &str) -> Option<&ViewModel> {
        self.collection.widgets.get(self.id)?.get(name)
    }

    pub fn widget_view_model_mut(&mut self, name: &str) -> Option<&mut ViewModel> {
        self.collection.widgets.get_mut(self.id)?.get_mut(name)
    }

    pub fn hierarchy_view_model(&self, name: &str) -> Option<&ViewModel> {
        self.collection
            .widgets
            .iter()
            .filter_map(|(id, view_models)| {
                id.distance_to(self.id).ok().and_then(|distance| {
                    if distance <= 0 {
                        Some((distance, view_models.get(name)?))
                    } else {
                        None
                    }
                })
            })
            .min_by(|(a, _), (b, _)| a.cmp(b))
            .map(|(_, view_model)| view_model)
    }

    pub fn hierarchy_view_model_mut(&mut self, name: &str) -> Option<&mut ViewModel> {
        self.collection
            .widgets
            .iter_mut()
            .filter_map(|(id, view_models)| {
                id.distance_to(self.id).ok().and_then(|distance| {
                    if distance <= 0 {
                        Some((distance, view_models.get_mut(name)?))
                    } else {
                        None
                    }
                })
            })
            .min_by(|(a, _), (b, _)| a.cmp(b))
            .map(|(_, view_model)| view_model)
    }
}

pub struct ViewModelObject<'a, T> {
    access: ValueWriteAccess<'a, T>,
    notifier: ViewModelNotifier,
}

impl<T> ViewModelObject<'_, T> {
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

impl<T> Deref for ViewModelObject<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.access
    }
}

impl<T> DerefMut for ViewModelObject<'_, T> {
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

    pub fn consume(self) -> T {
        self.value
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
        let view_model = ViewModel::produce(|properties| Foo {
            counter: ViewModelValue::new(0, properties.notifier(COUNTER_PROPERTY)),
            flag: false,
            flag_notifier: properties.notifier(FLAG_PROPERTY),
        });
        // handle to view-model data we can use to share around.
        // it stays alive as long as its view-model object.
        let handle = view_model.lazy::<Foo>().unwrap();
        collection.insert(FOO_VIEW_MODEL.to_owned(), view_model);

        // unbound properties won't trigger notification until we bind widgets to them.
        assert!(!collection.consume_notified_common_root().is_valid());
        handle.write().unwrap().toggle();
        assert!(!collection.consume_notified_common_root().is_valid());
        assert!(
            collection
                .get_mut(FOO_VIEW_MODEL)
                .unwrap()
                .properties
                .bindings(COUNTER_PROPERTY)
                .unwrap()
                .is_notified()
        );
        assert!(
            collection
                .get_mut(FOO_VIEW_MODEL)
                .unwrap()
                .properties
                .bindings(FLAG_PROPERTY)
                .unwrap()
                .is_notified()
        );

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
