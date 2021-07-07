use crate::Mutex;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::vec::Vec;
use derive_more::Display;

struct Channel<T> {
    deque: VecDeque<T>,
    sender_count: usize,
    receiver_gone: bool,
}

impl<T> Channel<T> {
    fn new() -> Self {
        Self::with_capacity(4)
    }

    fn with_capacity(cap: usize) -> Self {
        Self {
            deque: VecDeque::with_capacity(cap),
            sender_count: 1,
            receiver_gone: false,
        }
    }
}

type ArcCh<T> = Arc<Mutex<Channel<T>>>;
pub struct Sender<T>(ArcCh<T>);

#[derive(Display, Debug)]
pub enum SendError {
    #[display(fmt = "The receiver of the channel has gone")]
    ReceiverGone,
}

impl<T> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), SendError> {
        let mut ch = self.0.lock();
        if ch.receiver_gone {
            Err(SendError::ReceiverGone)
        } else {
            ch.deque.push_back(value);
            // TODO.kevin: awake the receiver task
            Ok(())
        }
    }

    pub fn clear(&self) -> usize {
        let mut ch = self.0.lock();
        ch.deque.drain(..).count()
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Sender<T> {
        self.0.lock().sender_count += 1;
        Sender(self.0.clone())
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut inner = self.0.lock();
        inner.sender_count -= 1;
        if inner.sender_count == 0 {
            // TODO.kevin: awake the receiver task
        }
    }
}

pub struct Receiver<T>(ArcCh<T>);

#[derive(Display, Debug)]
pub enum ReceiveError {
    #[display(fmt = "All senders of the channel have gone")]
    SenderGone,
}

impl<T> Receiver<T> {
    #[allow(clippy::should_implement_trait)]
    pub fn try_next(&mut self) -> Result<Option<T>, ReceiveError> {
        let mut ch = self.0.lock();
        if let Some(value) = ch.deque.pop_front() {
            return Ok(Some(value));
        } else if ch.sender_count == 0 {
            return Err(ReceiveError::SenderGone);
        }
        Ok(None)
    }

    pub fn drain(&mut self) -> impl Iterator<Item = T> {
        let mut ch = self.0.lock();
        ch.deque.drain(..).collect::<Vec<_>>().into_iter()
    }

    pub fn clear(&mut self) {
        let _ = self.0.lock().deque.drain(..);
    }
}

impl<T: Seq> Receiver<T> {
    pub fn peek_ind(&self) -> Result<Option<u64>, ReceiveError> {
        let ch = self.0.lock();
        if let Some(value) = ch.deque.get(0) {
            return Ok(Some(value.seq()));
        } else if ch.sender_count == 0 {
            return Err(ReceiveError::SenderGone);
        }
        Ok(None)
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.0.lock().receiver_gone = true;
    }
}

pub fn channel<T>() -> (Receiver<T>, Sender<T>) {
    let ch = Arc::new(Mutex::new(Channel::new()));
    let rx = Receiver(ch.clone());
    let tx = Sender(ch);
    (rx, tx)
}

pub trait Seq {
    fn seq(&self) -> u64;
}
