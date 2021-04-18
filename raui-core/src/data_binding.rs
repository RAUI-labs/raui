use crate::{application::ChangeNotifier, messenger::MessageData, props::PropsData, Prefab};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DataBinding<T>
where
    T: std::fmt::Debug + Default + Send + Sync,
{
    #[serde(skip)]
    data: Arc<RwLock<T>>,
    #[serde(skip)]
    notifier: Option<ChangeNotifier>,
}

impl<T> DataBinding<T>
where
    T: std::fmt::Debug + Default + Send + Sync,
{
    pub fn new(data: T) -> Self {
        Self {
            data: Arc::new(RwLock::new(data)),
            notifier: None,
        }
    }

    pub fn new_bound(data: T, notifier: ChangeNotifier) -> Self {
        Self {
            data: Arc::new(RwLock::new(data)),
            notifier: Some(notifier),
        }
    }

    pub fn bind(&mut self, notifier: ChangeNotifier) {
        self.notifier = Some(notifier);
    }

    pub fn unbind(&mut self) {
        self.notifier = None;
    }

    pub fn access<F, R>(&self, mut f: F) -> Option<R>
    where
        F: FnMut(&T) -> R,
    {
        if let Ok(data) = self.data.read() {
            Some(f(&data))
        } else {
            None
        }
    }

    pub fn read_cloned(&self) -> Option<T>
    where
        T: Clone,
    {
        if let Ok(data) = self.data.read() {
            let data: &T = &data;
            Some(data.clone())
        } else {
            None
        }
    }

    pub fn read_cloned_or_default(&self) -> T
    where
        T: Clone,
    {
        self.read_cloned().unwrap_or_default()
    }

    pub fn mutate<F, R>(&mut self, mut f: F) -> Option<R>
    where
        F: FnMut(&mut T) -> R,
    {
        if let Ok(mut data) = self.data.write() {
            let result = f(&mut data);
            if let Some(notifier) = self.notifier.as_mut() {
                notifier.change();
            }
            Some(result)
        } else {
            None
        }
    }

    pub fn write(&mut self, v: T) {
        if let Ok(mut data) = self.data.write() {
            *data = v;
            if let Some(notifier) = self.notifier.as_mut() {
                notifier.change();
            }
        }
    }
}

impl<T> PropsData for DataBinding<T>
where
    Self: Clone,
    T: 'static + std::fmt::Debug + Default + Send + Sync,
{
    fn clone_props(&self) -> Box<dyn PropsData> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl<T> Prefab for DataBinding<T> where T: std::fmt::Debug + Default + Send + Sync {}

impl<T> MessageData for DataBinding<T>
where
    Self: Clone,
    T: 'static + std::fmt::Debug + Default + Send + Sync,
{
    fn clone_message(&self) -> Box<dyn MessageData> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
