use crate::{application::Application, props::PropsData, Prefab};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DataBinding<T>
where
    T: std::fmt::Debug + Default + Send + Sync,
{
    #[serde(skip)]
    data: Arc<RwLock<T>>,
}

impl<T> DataBinding<T>
where
    T: std::fmt::Debug + Default + Send + Sync,
{
    pub fn new(data: T) -> Self {
        Self {
            data: Arc::new(RwLock::new(data)),
        }
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
            Some(f(&mut data))
        } else {
            None
        }
    }

    pub fn write(&mut self, v: T) {
        if let Ok(mut data) = self.data.write() {
            *data = v;
        }
    }

    pub fn mutate_synced<F, R>(&mut self, app: &mut Application, f: F) -> Option<R>
    where
        F: FnMut(&mut T) -> R,
    {
        self.sync(app);
        self.mutate(f)
    }

    pub fn write_synced(&mut self, app: &mut Application, v: T) {
        self.sync(app);
        self.write(v);
    }

    // For now we just mark application dirty - that will force it to re-render widgets next frame.
    pub fn sync(&self, app: &mut Application) {
        app.mark_dirty()
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
