use crate::widget::WidgetId;
use std::{
    any::Any,
    sync::mpsc::{Receiver, Sender},
};

pub type Signal = (WidgetId, Box<dyn Any>);

pub struct SignalReceiver(Receiver<Signal>);

impl SignalReceiver {
    pub fn new(receiver: Receiver<Signal>) -> Self {
        Self(receiver)
    }

    pub fn read(&self) -> Option<Signal> {
        match self.0.try_recv() {
            Ok(signal) => Some(signal),
            _ => None,
        }
    }

    pub fn read_all(&self) -> Vec<Signal> {
        let mut result = vec![];
        while let Ok(signal) = self.0.try_recv() {
            result.push(signal);
        }
        result
    }
}

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
        T: 'static + Any,
    {
        self.sender
            .send((self.id.clone(), Box::new(message)))
            .is_ok()
    }

    pub fn write_raw(&self, message: Box<dyn Any>) -> bool {
        self.sender.send((self.id.clone(), message)).is_ok()
    }

    pub fn write_raw_all<I>(&self, messages: I)
    where
        I: IntoIterator<Item = Box<dyn Any>>,
    {
        for data in messages {
            drop(self.sender.send((self.id.clone(), data)));
        }
    }
}
