pub use {
    dispatcher::{subscribe_default, using as using_dispatcher},
    send_mq::{default_send_mq, using as using_send_mq},
};

mod send_mq {
    use crate::MessageSendQueue;

    environmental::environmental!(global_send_mq: MessageSendQueue);

    pub fn using<F, R>(mq: &mut MessageSendQueue, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        global_send_mq::using(mq, f)
    }

    pub fn default_send_mq() -> MessageSendQueue {
        global_send_mq::with(|mq| mq.clone()).unwrap_or_else(|| {
            panic!("default_send_mq is called without using a global_send_mq");
        })
    }
}

mod dispatcher {
    use crate::{Message, MessageDispatcher, Path, dispatcher::Receiver};

    environmental::environmental!(global_dispatcher: MessageDispatcher);

    pub fn using<F, R>(mq: &mut MessageDispatcher, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        global_dispatcher::using(mq, f)
    }

    fn with<F: FnOnce(&mut MessageDispatcher) -> R, R>(f: F) -> Option<R> {
        global_dispatcher::with(f)
    }

    pub fn subscribe_default(path: impl Into<Path>) -> Receiver<Message> {
        with(move |dispatcher| dispatcher.subscribe(path))
            .expect("subscribe_default_typed called without using a global dispatcher")
    }
}
