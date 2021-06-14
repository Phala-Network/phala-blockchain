use phala_mq::{Message, MessageDispatcher, MessageSendQueue, Signer};

struct TestSigner(Vec<u8>);

impl Signer for TestSigner {
    fn sign(&self, _sequence: u64, message: &phala_mq::Message) -> Vec<u8> {
        let mut sig = self.0.clone();
        sig.extend(message.payload.iter());
        sig
    }
}

#[test]
fn test_send_message() {
    let queue = MessageSendQueue::new();
    let runtime = b"r0".to_vec();
    let contract1 = b"contract1".to_vec();

    {
        let signer = TestSigner(b"key0".to_vec());

        let handle00 = queue.create_handle(runtime.clone(), signer);

        handle00.send(b"payload00".to_vec(), b"phala.network/test0".to_vec());

        let signer = TestSigner(b"key1".to_vec());

        let handle01 = queue.create_handle(runtime.clone(), signer);
        handle01.send(b"payload01".to_vec(), b"phala.network/test1".to_vec());

        handle00.send(b"payload02".to_vec(), b"phala.network/test1".to_vec());

        let messages = queue.all_messages();

        assert_eq!(messages.len(), 3);

        assert_eq!(messages[0].message.sender, runtime);
        assert_eq!(messages[0].sequence, 0);
        assert_eq!(messages[0].signature, b"key0payload00");

        assert_eq!(messages[1].message.sender, runtime);
        assert_eq!(messages[1].sequence, 1);
        assert_eq!(messages[1].signature, b"key1payload01");

        assert_eq!(messages[2].message.sender, runtime);
        assert_eq!(messages[2].sequence, 2);
        assert_eq!(messages[2].signature, b"key0payload02");
    }

    {
        let signer = TestSigner(b"a key".to_vec());
        let handle = queue.create_handle(contract1.clone(), signer);

        handle.send(b"energy".to_vec(), b"/the/hole".to_vec());
        handle.send(b"energy".to_vec(), b"/the/hole".to_vec());
        handle.send(b"energy".to_vec(), b"/the/hole".to_vec());

        assert_eq!(queue.messages(&&contract1).len(), 3);
    }


    {
        queue.purge(|sender| {
            match &sender[..] {
                b"r0" => {
                    1
                }
                _ => {
                    0
                }
            }
        });

        let runtime_msgs = queue.messages(&runtime);
        let contract1_msgs = queue.messages(&contract1);

        assert_eq!(runtime_msgs.len(), 2);
        assert_eq!(contract1_msgs.len(), 3);
    }
}

#[test]
fn test_dispatcher() {
    let mut dispatcher = MessageDispatcher::new();

    let mut sub0 = dispatcher.subscribe(*b"path0");
    let mut sub1 = dispatcher.subscribe(*b"path1");

    let n = dispatcher.dispatch(Message::new(*b"0", *b"path0", b"payload0".to_vec()));
    assert_eq!(n, 1);

    let mut sub2 = dispatcher.subscribe(*b"path0");
    let n = dispatcher.dispatch(Message::new(*b"0", *b"path1", b"payload1".to_vec()));
    assert_eq!(n, 1);
    let _ = dispatcher.dispatch(Message::new(*b"1", *b"path1", b"payload2".to_vec()));
    let n = dispatcher.dispatch(Message::new(*b"1", *b"path0", b"payload3".to_vec()));
    assert_eq!(n, 2);

    {
        let msgs: Vec<Message> = sub0.drain().collect();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].sender, b"0");
        assert_eq!(msgs[0].destination, b"path0");
        assert_eq!(msgs[0].payload, b"payload0");
        assert_eq!(msgs[1].sender, b"1");
        assert_eq!(msgs[1].destination, b"path0");
        assert_eq!(msgs[1].payload, b"payload3");
    }
    {
        let msgs: Vec<Message> = sub1.drain().collect();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].sender, b"0");
        assert_eq!(msgs[0].destination, b"path1");
        assert_eq!(msgs[0].payload, b"payload1");
        assert_eq!(msgs[1].sender, b"1");
        assert_eq!(msgs[1].destination, b"path1");
        assert_eq!(msgs[1].payload, b"payload2");
    }
    {
        let msgs: Vec<Message> = sub2.drain().collect();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].sender, b"1");
        assert_eq!(msgs[0].destination, b"path0");
        assert_eq!(msgs[0].payload, b"payload3");
    }
    
}
