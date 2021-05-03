use crate::{
    messenger::{Message, MessageData},
    widget::WidgetId,
};
use std::sync::mpsc::Sender;

pub type Signal = (WidgetId, Box<dyn MessageData>);

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
        T: 'static + MessageData,
    {
        self.sender
            .send((self.id.clone(), Box::new(message)))
            .is_ok()
    }

    pub fn write_raw(&self, message: Message) -> bool {
        self.sender.send((self.id.clone(), message)).is_ok()
    }

    pub fn write_raw_all<I>(&self, messages: I)
    where
        I: IntoIterator<Item = Message>,
    {
        for data in messages {
            let _ = self.sender.send((self.id.clone(), data));
        }
    }
}
