//! Widget state types

use crate::props::{Props, PropsData, PropsError};
use std::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub enum StateError {
    Props(PropsError),
    CouldNotWriteData,
}

#[derive(Clone)]
pub struct StateUpdate(Sender<Props>);

impl StateUpdate {
    pub fn new(sender: Sender<Props>) -> Self {
        Self(sender)
    }

    pub fn write<T>(&self, data: T) -> Result<(), StateError>
    where
        T: Into<Props>,
    {
        if self.0.send(data.into()).is_err() {
            Err(StateError::CouldNotWriteData)
        } else {
            Ok(())
        }
    }
}

pub struct State<'a> {
    data: &'a Props,
    update: StateUpdate,
}

impl<'a> State<'a> {
    pub fn new(data: &'a Props, update: StateUpdate) -> Self {
        Self { data, update }
    }

    #[inline]
    pub fn data(&self) -> &Props {
        self.data
    }

    pub fn has<T>(&self) -> bool
    where
        T: 'static + PropsData,
    {
        self.data.has::<T>()
    }

    pub fn read<T>(&self) -> Result<&'a T, StateError>
    where
        T: 'static + PropsData,
    {
        match self.data.read() {
            Ok(v) => Ok(v),
            Err(e) => Err(StateError::Props(e)),
        }
    }

    pub fn map_or_default<T, R, F>(&self, f: F) -> R
    where
        T: 'static + PropsData,
        R: Default,
        F: FnMut(&T) -> R,
    {
        self.data.map_or_default(f)
    }

    pub fn map_or_else<T, R, F, E>(&self, f: F, e: E) -> R
    where
        T: 'static + PropsData,
        F: FnMut(&T) -> R,
        E: FnMut() -> R,
    {
        self.data.map_or_else(f, e)
    }

    pub fn read_cloned<T>(&self) -> Result<T, StateError>
    where
        T: 'static + PropsData + Clone,
    {
        match self.data.read_cloned() {
            Ok(v) => Ok(v),
            Err(e) => Err(StateError::Props(e)),
        }
    }

    pub fn read_cloned_or_default<T>(&self) -> T
    where
        T: 'static + PropsData + Clone + Default,
    {
        self.data.read_cloned_or_default()
    }

    pub fn read_cloned_or_else<T, F>(&self, f: F) -> T
    where
        T: 'static + PropsData + Clone + Default,
        F: FnMut() -> T,
    {
        self.data.read_cloned_or_else(f)
    }

    pub fn write<T>(&self, data: T) -> Result<(), StateError>
    where
        T: 'static + PropsData + Send + Sync,
    {
        self.update.write(data)
    }

    pub fn write_with<T>(&self, data: T) -> Result<(), StateError>
    where
        T: 'static + PropsData + Send + Sync,
    {
        self.update.write(self.data.to_owned().with(data))
    }

    pub fn write_without<T>(&self) -> Result<(), StateError>
    where
        T: 'static + PropsData + Send + Sync,
    {
        self.update.write(self.data.to_owned().without::<T>())
    }

    pub fn mutate<T, F>(&self, mut f: F) -> Result<(), StateError>
    where
        T: 'static + PropsData + Send + Sync,
        F: FnMut(&T) -> T,
    {
        match self.read() {
            Ok(data) => {
                let data = f(data);
                self.write(data)
            }
            Err(error) => Err(error),
        }
    }

    pub fn mutate_cloned<T, F>(&self, mut f: F) -> Result<(), StateError>
    where
        T: 'static + PropsData + Send + Sync + Clone,
        F: FnMut(&mut T),
    {
        match self.read::<T>() {
            Ok(data) => {
                let mut data = data.clone();
                f(&mut data);
                self.write(data)
            }
            Err(error) => Err(error),
        }
    }

    pub fn update(&self) -> &StateUpdate {
        &self.update
    }
}
