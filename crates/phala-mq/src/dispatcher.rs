use core::marker::PhantomData;

use alloc::{collections::BTreeMap, vec::Vec};

use crate::simple_mpsc::{channel, ReceiveError, Receiver as RawReceiver, Sender, Seq};
use crate::types::{Message, Path};
use crate::{BindTopic, MessageOrigin};
use derive_more::Display;
use parity_scale_codec::{Decode, Error as CodecError};

impl Seq for (u64, Message) {
    fn seq(&self) -> u64 {
        self.0
    }
}

#[derive(Default)]
pub struct MessageDispatcher {
    subscribers: BTreeMap<Path, Vec<Sender<(u64, Message)>>>,
    local_index: u64,
    //match_subscribers: Vec<Matcher, Vec<Sender<Message>>>,
}

pub type Receiver<T> = RawReceiver<(u64, T)>;

impl MessageDispatcher {
    pub fn new() -> Self {
        MessageDispatcher {
            subscribers: Default::default(),
            local_index: 0,
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
        self.subscribe(<T as BindTopic>::topic()).into()
    }

    /// Dispatch a message.
    /// Returns number of receivers dispatched to.
    pub fn dispatch(&mut self, message: Message) -> usize {
        let mut count = 0;
        let sn = self.local_index;
        self.local_index += 1;
        if let Some(receivers) = self.subscribers.get_mut(message.destination.path()) {
            receivers.retain(|receiver| {
                if let Err(error) = receiver.send((sn, message.clone())) {
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

    pub fn reset_local_index(&mut self) {
        self.local_index = 0;
    }

    /// Drop all unhandled messages.
    pub fn clear(&mut self) -> usize {
        let mut count = 0;
        for subscriber in self.subscribers.values_mut().flatten() {
            count += subscriber.clear();
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

pub struct TypedReceiver<T> {
    queue: Receiver<Message>,
    _t: PhantomData<T>,
}

impl<T: Decode> TypedReceiver<T> {
    pub fn try_next(&mut self) -> Result<Option<(u64, T, MessageOrigin)>, TypedReceiveError> {
        let message = self.queue.try_next().map_err(|e| match e {
            ReceiveError::SenderGone => TypedReceiveError::SenderGone,
        })?;
        let (sn, msg) = match message {
            None => return Ok(None),
            Some(m) => m,
        };
        let typed = Decode::decode(&mut &msg.payload[..])?;
        Ok(Some((sn, typed, msg.sender)))
    }

    pub fn peek_ind(&self) -> Result<Option<u64>, ReceiveError> {
        self.queue.peek_ind()
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

#[macro_export]
macro_rules! select {
    (
        $( $bind:pat = $mq:expr => $block:expr, )+
    ) => {{
        let mut min = None;
        let mut min_ind = 0;
        let mut ind = 0;
        $({
            match $mq.peek_ind() {
                Ok(Some(sn)) => match min {
                    None => { min = Some(sn); min_ind = ind; }
                    Some(old) if sn < old => { min = Some(sn); min_ind = ind; }
                    _ => (),
                },
                Err(_) => { min = Some(0); min_ind = ind; }
                _ => (),
            }
            ind += 1;
        })+

        let mut ind = 0;
        let mut rv = None;
        if min.is_some() {
            $({
                if min_ind == ind {
                    let msg = $mq.try_next().transpose();
                    rv = match msg {
                        Some($bind) => Some($block),
                        None => None
                    };
                }
                ind += 1;
            })+
        }
        rv
    }};
}


#[macro_export]
macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}


#[macro_export]
macro_rules! select_ignore_errors {
    (
        $( $bind:pat = $mq:expr => $block:expr, )+
    ) => {{
        $crate::select! {
            $(
                message = $mq => match message {
                    Ok(msg) => {
                        let $bind = (msg.1, msg.2);
                        {
                            $block
                        }
                    }
                    Err(err) => {
                        log::warn!("[{}] mq ignored error: {:?}", $crate::function!(), err);
                    }
                },
            )+
        }
    }}
}
