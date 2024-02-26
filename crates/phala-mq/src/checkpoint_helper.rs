pub use {
    dispatcher::{subscribe_default, using as using_dispatcher},
    send_mq::{global_send_mq, using as using_send_mq},
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

    pub fn global_send_mq() -> MessageSendQueue {
        global_send_mq::with(|mq| mq.clone())
            .expect("global_send_mq is called without using a global_send_mq")
    }
}

mod dispatcher {
    use crate::{dispatcher::Receiver, Message, MessageDispatcher, Path};

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
            .expect("subscribe_default called without using a global dispatcher")
    }
}

#[test]
fn it_works() {
    use parity_scale_codec::{Decode, Encode};

    crate::bind_topic!(TestMessage, b"topic0");
    #[derive(Decode, Encode)]
    struct TestMessage {
        value: u32,
    }

    let mut mq = crate::MessageSendQueue::new();
    let mut dispatcher = crate::MessageDispatcher::new();
    let mut rx = dispatcher.subscribe_bound::<TestMessage>();

    let sealed = serde_cbor::to_vec(&rx).unwrap();
    let mut recovered_rx = using_send_mq(&mut mq, || {
        using_dispatcher(&mut dispatcher, || {
            let mut ingress: crate::dispatcher::TypedReceiver<TestMessage> =
                serde_cbor::from_reader(&sealed[..]).unwrap();
            assert!(ingress.try_next().unwrap().is_none());
            let _egress = global_send_mq();
            ingress
        })
    });

    dispatcher.dispatch(crate::Message::new(
        crate::MessageOrigin::Gatekeeper,
        "topic0",
        TestMessage { value: 42 }.encode(),
    ));
    assert_eq!(rx.try_next().unwrap().unwrap().1.value, 42);
    assert_eq!(recovered_rx.try_next().unwrap().unwrap().1.value, 42);
}
