pub use {
    dispatcher::{subscribe_bound_default, subscribe_default_typed, using as using_dispatcher},
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
    use crate::{BindTopic, MessageDispatcher, Path, TypedReceiver};
    use parity_scale_codec::Decode;

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

    pub fn subscribe_bound_default<T: BindTopic + Decode>() -> TypedReceiver<T> {
        with(|dispatcher| dispatcher.subscribe_bound())
            .expect("subscribe_bound_default called without using a global dispatcher")
    }

    pub fn subscribe_default_typed<T: Decode>(path: impl Into<Path>) -> TypedReceiver<T> {
        with(move |dispatcher| dispatcher.subscribe(path).into())
            .expect("subscribe_default_typed called without using a global dispatcher")
    }
}
