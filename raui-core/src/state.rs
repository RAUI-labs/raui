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

    pub fn read<T: 'static>(&self) -> Result<&'a T, StateError> {
        if let Some(data) = self.data.downcast_ref::<T>() {
            Ok(data)
        } else {
            Err(StateError::CouldNotReadData)
        }
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
