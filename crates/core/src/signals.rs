//! Widget signals
//!
//! Signals are a way for widgets to send [messages][crate::messenger] to the RAUI
//! [`Application`][crate::application::Application]. This can be used to create custom integrations
//! with the RAUI host or rendering backend.
//!
//! Signals may be sent using the [`SignalSender`] in the widget [change context][change_context] or
//! [unmount context][unmount_context].
//!
//! [change_context]: crate::widget::context::WidgetMountOrChangeContext
//!
//! [unmount_context]: crate::widget::context::WidgetUnmountContext

use crate::{
    messenger::{Message, MessageData},
    widget::WidgetId,
};
use std::sync::mpsc::Sender;

/// A signal is a [message][crate::messenger] sent by a widget that can be read by the
/// [`Application`][crate::application::Application]
pub type Signal = (WidgetId, Box<dyn MessageData>);

/// Used to send [`Signal`]s from a component [change context][change_context]
///
/// [change_context]: crate::widget::context::WidgetMountOrChangeContext
#[derive(Clone)]
pub struct SignalSender {
    id: WidgetId,
    sender: Sender<Signal>,
}

impl SignalSender {
    /// Create a new [`SignalSender`]
    pub(crate) fn new(id: WidgetId, sender: Sender<Signal>) -> Self {
        Self { id, sender }
    }

    /// Send a message
    ///
    /// Returns `false` if the message could not successfully be sent
    pub fn write<T>(&self, message: T) -> bool
    where
        T: 'static + MessageData,
    {
        self.sender
            .send((self.id.clone(), Box::new(message)))
            .is_ok()
    }

    /// Send a raw [`Message`]
    ///
    /// Returns `false` if the message could not be successfully sent
    pub fn write_raw(&self, message: Message) -> bool {
        self.sender.send((self.id.clone(), message)).is_ok()
    }

    /// Sends a set of raw [`Message`]s from an iterator
    pub fn write_raw_all<I>(&self, messages: I)
    where
        I: IntoIterator<Item = Message>,
    {
        for data in messages {
            let _ = self.sender.send((self.id.clone(), data));
        }
    }
}
