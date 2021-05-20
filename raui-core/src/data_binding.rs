//! Mechanism for integrating external data into the UI
//!
//! [`DataBinding`]s are a way to take data external to the RAUI UI and make sure that the UI is
//! updated every time that data changes. [`DataBinding`]s can be created and then added as
//! [`Props`][crate::props] to widgets.
//!
//! The specifics of how to create [`DataBinding`]s will probably vary depending on the RAUI
//! renderer and integration crate you are using, but below we show an example of using the
//! lower-level mechanisms to create a [`DataBinding`] directly.
//!
//! # Example
//!
//! ```
//! # use raui_core::prelude::*;
//! # fn app_component(_: WidgetContext) -> WidgetNode { widget!(()) }
//! /// This is our external data type we want to bind into our RAUI UI
//! #[derive(Debug, Clone, Default)]
//! struct MyGlobalData {
//!     /// A simple counter value
//!     counter: i32,
//! }
//!
//! // Our external data
//! let data = MyGlobalData {
//!     counter: 0,
//! };
//!
//! // Create our application
//! let mut app = Application::new();
//!
//! // Get the change notifier of the application
//! let change_notifier = app.change_notifier();
//!
//! // Create a data binding for our global data bound to our app's change
//! // notifier.
//! let data_binding = DataBinding::new_bound(data, change_notifier);
//!
//! // Create our app properties and add our data binding to it
//! let app_props = Props::new(data_binding);
//!
//! // Create our widget tree, including our app component and it's props
//! let tree = widget! {
//!     (app_component: {app_props})
//! };
//! ```

use crate::{application::ChangeNotifier, messenger::MessageData, props::PropsData, Prefab};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

/// Wraps internal data and optionally notifies an [`Application`][crate::application::Application]
/// of changes to it
/// 
/// See [module docs][self] for a full example.
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
    /// Create a new [`DataBinding`] that wraps `data`
    ///
    /// It will create the data as _unbound_ meaning that changes to the data will not notify any
    /// [`Application`][crate::application::Application]. The [`DataBinding`] can afterwards be
    /// _bound_ with the [`bind`][Self::bind] method.
    pub fn new(data: T) -> Self {
        Self {
            data: Arc::new(RwLock::new(data)),
            notifier: None,
        }
    }

    /// Create a new [`DataBinding`] that wraps `data` and notifies an
    /// [`Application`][crate::application::Application]'s [`ChangeNotifier`] when the data is
    /// mutated.
    pub fn new_bound(data: T, notifier: ChangeNotifier) -> Self {
        Self {
            data: Arc::new(RwLock::new(data)),
            notifier: Some(notifier),
        }
    }

    /// Bind this [`DataBinding`] to a [`ChangeNotifier`]
    pub fn bind(&mut self, notifier: ChangeNotifier) {
        self.notifier = Some(notifier);
    }

    /// Unbind this [`DataBinding`] so that changes to the data no longer trigger application
    /// updates
    pub fn unbind(&mut self) {
        self.notifier = None;
    }

    /// Access the inner data of the binding inside of the provided closure
    ///
    /// This will return [`None`] and will **not** run the supplied closure if a lock to the inner
    /// data cannot be obtained due to lock [poisoning][RwLock#poisoning].
    ///
    /// # Example
    ///
    /// ```
    /// # use raui_core::prelude::*;
    /// let binding = DataBinding::new(false);
    ///
    /// let x = binding.access(|data| {
    ///     // Return the opposite of what's in our data
    ///     !data
    /// });
    ///
    /// assert_eq!(x, Some(true));
    /// ```
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

    /// Get a clone of the inner data
    ///
    /// Returns [`None`] if the internal lock to the inner data has been
    /// [poisoned][RwLock#poisoning].
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

    /// Attempt to obtain a clone of the inner data or otherwise return the type's default value
    pub fn read_cloned_or_default(&self) -> T
    where
        T: Clone,
    {
        self.read_cloned().unwrap_or_default()
    }

    /// Use a closure to mutate the inner data and notify the [`ChangeNotifier`] ( if set )
    ///
    /// This will return [`None`] and will **not** run the supplied closure if a lock to the inner
    /// data cannot be obtained due to lock [poisoning][RwLock#poisoning].
    ///
    /// # Example
    ///
    /// ```
    /// # use raui_core::prelude::*;
    /// let mut binding = DataBinding::new(false);
    ///
    /// let x = binding.mutate(|data| {
    ///     // Update the data
    ///     *data = true;
    ///
    ///     *data
    /// });
    ///
    /// assert_eq!(x, Some(true));
    /// assert_eq!(binding.read_cloned(), Some(true));
    /// ```
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

    /// Set the inner data directly and notify the [`ChangeNotifier`] ( if set )
    pub fn write(&mut self, v: T) {
        // TODO: This will silently swallow any failure to obtain the RwLock, should we return a
        // result instead?
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
