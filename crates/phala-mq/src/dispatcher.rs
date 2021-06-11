use alloc::{collections::BTreeMap, vec::Vec};

use crate::simple_mpsc::{channel, Receiver, Sender};
use crate::types::{Message, Path};

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

    /// Dispatch a message.
    /// Returns number of receivers dispatched to.
    pub fn dispatch(&mut self, message: Message) -> usize {
        let mut count = 0;
        if let Some(receivers) = self.subscribers.get_mut(&message.destination) {
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
