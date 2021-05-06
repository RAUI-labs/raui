//! Widget messaging

use crate::widget::WidgetId;
use std::{any::Any, sync::mpsc::Sender};

pub trait MessageData: std::fmt::Debug + Send + Sync {
    fn clone_message(&self) -> Box<dyn MessageData>;
    fn as_any(&self) -> &dyn Any;
}

impl Clone for Box<dyn MessageData> {
    fn clone(&self) -> Self {
        self.clone_message()
    }
}

pub type Message = Box<dyn MessageData>;
pub type Messages = Vec<Message>;

#[derive(Clone)]
pub struct MessageSender(Sender<(WidgetId, Message)>);

impl MessageSender {
    pub fn new(sender: Sender<(WidgetId, Message)>) -> Self {
        Self(sender)
    }

    pub fn write<T>(&self, id: WidgetId, message: T) -> bool
    where
        T: 'static + MessageData,
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
            let _ = self.0.send(data);
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
        T: 'static + MessageData,
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

/// Macro for implementing [`MessageData`]. You may prefer to use the [derive
/// macro][`macro@crate::MessageData`] instead.
#[macro_export]
macro_rules! implement_message_data {
    ($type_name:ty) => {
        impl $crate::messenger::MessageData for $type_name
        where
            Self: Clone,
        {
            fn clone_message(&self) -> Box<dyn $crate::messenger::MessageData> {
                Box::new(self.clone())
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    };
}

implement_message_data!(());
implement_message_data!(i8);
implement_message_data!(i16);
implement_message_data!(i32);
implement_message_data!(i64);
implement_message_data!(i128);
implement_message_data!(u8);
implement_message_data!(u16);
implement_message_data!(u32);
implement_message_data!(u64);
implement_message_data!(u128);
implement_message_data!(f32);
implement_message_data!(f64);
implement_message_data!(bool);
implement_message_data!(String);
