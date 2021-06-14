use phala_mq::{Message, MessageDispatcher, MessageSendQueue, Signer};

struct TestSigner(Vec<u8>);

impl Signer for TestSigner {
    fn sign(&self, sequence: u64, message: &phala_mq::Message) -> Vec<u8> {
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
}
