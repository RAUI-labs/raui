use crate::widget::WidgetId;
use std::{
    any::Any,
    collections::{HashMap, VecDeque},
    sync::mpsc::{Receiver, Sender},
};

pub type Message = Box<dyn Any>;
pub type Messages = VecDeque<Message>;

pub struct MessageReceiver(Receiver<(WidgetId, Message)>);

impl MessageReceiver {
    pub fn new(receiver: Receiver<(WidgetId, Message)>) -> Self {
        Self(receiver)
    }

    pub fn process(&mut self) -> HashMap<WidgetId, Messages> {
        let mut result = HashMap::<WidgetId, Messages>::new();
        while let Ok((id, message)) = self.0.try_recv() {
            if let Some(list) = result.get_mut(&id) {
                list.push_back(message);
            } else {
                let mut list = Messages::with_capacity(1);
                list.push_back(message);
                result.insert(id, list);
            }
        }
        result
    }
}

#[derive(Clone)]
pub struct MessageSender(Sender<(WidgetId, Message)>);

impl MessageSender {
    pub fn new(sender: Sender<(WidgetId, Message)>) -> Self {
        Self(sender)
    }

    pub fn write(&self, id: WidgetId, message: Message) -> bool {
        self.0.send((id, message)).is_ok()
    }

    pub fn write_all<I>(&self, messages: I)
    where
        I: IntoIterator<Item = (WidgetId, Message)>,
    {
        for data in messages {
            drop(self.0.send(data));
        }
    }
}

pub struct Messenger {
    sender: MessageSender,
    messages: Messages,
}

impl Messenger {
    pub fn new(sender: MessageSender, messages: Messages) -> Self {
        Self { sender, messages }
    }

    pub fn read(&mut self) -> Option<Message> {
        self.messages.pop_front()
    }

    pub fn read_all(&mut self) -> Messages {
        std::mem::replace(&mut self.messages, Messages::new())
    }

    pub fn write(&self, id: WidgetId, message: Message) -> bool {
        self.sender.write(id, message)
    }
}
