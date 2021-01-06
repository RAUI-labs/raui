use crate::widget::WidgetId;
use std::{any::Any, sync::mpsc::Sender};

pub type Signal = (WidgetId, Box<dyn Any + Send + Sync>);

#[derive(Clone)]
pub struct SignalSender {
    id: WidgetId,
    sender: Sender<Signal>,
}

impl SignalSender {
    pub fn new(id: WidgetId, sender: Sender<Signal>) -> Self {
        Self { id, sender }
    }

    pub fn write<T>(&self, message: T) -> bool
    where
        T: 'static + Any + Send + Sync,
    {
        self.sender
            .send((self.id.clone(), Box::new(message)))
            .is_ok()
    }

    pub fn write_raw(&self, message: Box<dyn Any + Send + Sync>) -> bool {
        self.sender.send((self.id.clone(), message)).is_ok()
    }

    pub fn write_raw_all<I>(&self, messages: I)
    where
        I: IntoIterator<Item = Box<dyn Any + Send + Sync>>,
    {
        for data in messages {
            drop(self.sender.send((self.id.clone(), data)));
        }
    }
}
