use std::{any::Any, sync::mpsc::Sender};

pub enum StateError {
    CouldNotReadData,
    CouldNotWriteData,
}

pub type StateData = Box<dyn Any>;

#[derive(Clone)]
pub struct StateUpdate(Sender<StateData>);

impl StateUpdate {
    pub fn new(sender: Sender<StateData>) -> Self {
        Self(sender)
    }

    pub fn write<T>(&self, data: T) -> Result<(), StateError>
    where
        T: 'static,
    {
        if self.0.send(Box::new(data)).is_err() {
            Err(StateError::CouldNotWriteData)
        } else {
            Ok(())
        }
    }
}

pub struct State<'a> {
    data: &'a StateData,
    update: StateUpdate,
}

impl<'a> State<'a> {
    pub fn new(data: &'a StateData, update: StateUpdate) -> Self {
        Self { data, update }
    }

    pub fn read<T>(&self) -> Result<&'a T, StateError>
    where
        T: 'static,
    {
        if let Some(data) = self.data.downcast_ref::<T>() {
            Ok(data)
        } else {
            Err(StateError::CouldNotReadData)
        }
    }

    pub fn map_or_default<T, R, F>(&self, mut f: F) -> R
    where
        T: 'static,
        R: Default,
        F: FnMut(&T) -> R,
    {
        match self.read() {
            Ok(data) => f(data),
            Err(_) => R::default(),
        }
    }

    pub fn map_or_else<T, R, F, E>(&self, mut f: F, mut e: E) -> R
    where
        T: 'static,
        F: FnMut(&T) -> R,
        E: FnMut() -> R,
    {
        match self.read() {
            Ok(data) => f(data),
            Err(_) => e(),
        }
    }

    pub fn read_cloned<T>(&self) -> Result<T, StateError>
    where
        T: 'static + Clone,
    {
        self.read::<T>().map(|v| v.clone())
    }

    pub fn read_cloned_or_default<T>(&self) -> T
    where
        T: 'static + Clone + Default,
    {
        self.read_cloned().unwrap_or_default()
    }

    pub fn write<T>(&self, data: T) -> Result<(), StateError>
    where
        T: 'static,
    {
        self.update().write(data)
    }

    pub fn update(&self) -> &StateUpdate {
        &self.update
    }
}
