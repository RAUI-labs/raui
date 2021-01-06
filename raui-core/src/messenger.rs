use crate::widget::WidgetId;
use std::{any::Any, sync::mpsc::Sender};

pub type Message = Box<dyn Any + Send + Sync>;
pub type Messages = Vec<Message>;

#[derive(Clone)]
pub struct MessageSender(Sender<(WidgetId, Message)>);

impl MessageSender {
    pub fn new(sender: Sender<(WidgetId, Message)>) -> Self {
        Self(sender)
    }

    pub fn write<T>(&self, id: WidgetId, message: T) -> bool
    where
        T: 'static + Any + Send + Sync,
    {
        self.0.send((id, Box::new(message))).is_ok()
    }

    pub fn write_raw(&self, id: WidgetId, message: Message) -> bool {
        self.0.send((id, message)).is_ok()
    }

    pub fn write_raw_all<I>(&self, messages: I)
    where
        I: IntoIterator<Item = (WidgetId, Message)>,
    {
        for data in messages {
            drop(self.0.send(data));
        }
    }
}

pub struct Messenger<'a> {
    sender: MessageSender,
    pub messages: &'a [Message],
}

impl<'a> Messenger<'a> {
    pub fn new(sender: MessageSender, messages: &'a [Message]) -> Self {
        Self { sender, messages }
    }

    pub fn write<T>(&self, id: WidgetId, message: T) -> bool
    where
        T: 'static + Send + Sync,
    {
        self.sender.write(id, message)
    }

    pub fn write_raw(&self, id: WidgetId, message: Message) -> bool {
        self.sender.write_raw(id, message)
    }

    pub fn write_raw_all<I>(&self, messages: I)
    where
        I: IntoIterator<Item = (WidgetId, Message)>,
    {
        self.sender.write_raw_all(messages);
    }
}
