use core::marker::PhantomData;

use alloc::{collections::BTreeMap, vec::Vec};

use crate::simple_mpsc::{channel, ReceiveError, Receiver, Sender};
use crate::types::{Message, Path};
use crate::{BindTopic, MessageOrigin};
use derive_more::Display;
use parity_scale_codec::{Decode, Error as CodecError};

#[derive(Default)]
pub struct MessageDispatcher {
    subscribers: BTreeMap<Path, Vec<Sender<Message>>>,
    //match_subscribers: Vec<Matcher, Vec<Sender<Message>>>,
}

impl MessageDispatcher {
    pub fn new() -> Self {
        MessageDispatcher {
            subscribers: Default::default(),
        }
    }

    /// Subscribe messages which are sent to `path`.
    /// Returns a Receiver channel end.
    pub fn subscribe(&mut self, path: impl Into<Path>) -> Receiver<Message> {
        let (rx, tx) = channel();
        let entry = self.subscribers.entry(path.into()).or_default();
        entry.push(tx);
        rx
    }

    /// Subscribe messages which implementing BindTopic
    /// Returns a TypedReceiver channel end.
    pub fn subscribe_bound<T: Decode + BindTopic>(&mut self) -> TypedReceiver<T> {
        self.subscribe(<T as BindTopic>::TOPIC).into()
    }

    /// Dispatch a message.
    /// Returns number of receivers dispatched to.
    pub fn dispatch(&mut self, message: Message) -> usize {
        let mut count = 0;
        if let Some(receivers) = self.subscribers.get_mut(message.destination.path()) {
            receivers.retain(|receiver| {
                if let Err(error) = receiver.send(message.clone()) {
                    use crate::simple_mpsc::SendError::*;
                    match error {
                        ReceiverGone => false,
                    }
                } else {
                    count += 1;
                    true
                }
            });
        }
        count
    }
}

#[derive(Display, Debug)]
pub enum TypedReceiveError {
    #[display(fmt = "All senders of the channel have gone")]
    SenderGone,
    #[display(fmt = "Decode message failed: {}", _0)]
    CodecError(CodecError),
}

impl From<CodecError> for TypedReceiveError {
    fn from(e: CodecError) -> Self {
        Self::CodecError(e)
    }
}

pub struct TypedReceiver<T: Decode> {
    queue: Receiver<Message>,
    _t: PhantomData<T>,
}

impl<T: Decode> TypedReceiver<T> {
    pub fn try_next(&mut self) -> Result<Option<(T, MessageOrigin)>, TypedReceiveError> {
        let message = self.queue.try_next().map_err(|e| match e {
            ReceiveError::SenderGone => TypedReceiveError::SenderGone,
        })?;
        let Message {
            sender,
            destination: _,
            payload,
        } = match message {
            None => return Ok(None),
            Some(m) => m,
        };
        let typed = Decode::decode(&mut &payload[..])?;
        Ok(Some((typed, sender)))
    }
}

impl<T: Decode> From<Receiver<Message>> for TypedReceiver<T> {
    fn from(queue: Receiver<Message>) -> Self {
        Self {
            queue,
            _t: Default::default(),
        }
    }
}
