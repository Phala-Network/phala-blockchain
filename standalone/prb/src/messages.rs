use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use log::{debug, error};
use parity_scale_codec::Encode;
use phala_types::messaging::{MessageOrigin, SignedMessage};
use tokio::sync::mpsc;

use crate::{bus::Bus, tx::TxManager};

pub enum MessagesEvent {
    SyncMessages((u64, MessageOrigin, Vec<SignedMessage>)),
    Completed((MessageOrigin, u64)),
}

pub type MessagesRx = mpsc::UnboundedReceiver<MessagesEvent>;
pub type MessagesTx = mpsc::UnboundedSender<MessagesEvent>;

pub struct MessageContext {
}

pub struct SenderContext {
    sender: MessageOrigin,
    node_next_sequence: u64,
    pending_messages: HashMap<u64, MessageContext>,
}

impl SenderContext {
    pub fn calculate_next_sequence(&self) -> u64 {
        let mut next_sequence = self.node_next_sequence;
        while self.pending_messages.contains_key(&next_sequence) {
            next_sequence += 1;
        }
        next_sequence
    }
}

pub async fn master_loop(
    mut rx: MessagesRx,
    bus: Arc<Bus>,
    txm: Arc<TxManager>,
) -> Result<()> {
    let mut sender_contexts = HashMap::<MessageOrigin, SenderContext>::new();

    loop {
        let event = rx.recv().await;
        if event.is_none() {
            break
        }

        let event = event.unwrap();
        match event {
            MessagesEvent::SyncMessages((pool_id, sender, messages)) => {
                let sender_context = sender_contexts
                    .entry(sender.clone())
                    .or_insert_with(|| SenderContext {
                        sender: sender.clone(),
                        node_next_sequence: 0,
                        pending_messages: HashMap::new(),
                    });

                for message in messages {
                    let next_sequence = sender_context.calculate_next_sequence();
                    if message.sequence != next_sequence {
                        debug!("[{}] Ignoring #{} message since not matching next_sequence {}.", sender, message.sequence, next_sequence);
                        continue;
                    }

                    let _message_context = sender_context.pending_messages
                        .entry(message.sequence)
                        .or_insert_with(|| MessageContext {
                        });

                    let bus = bus.clone();
                    let txm = txm.clone();
                    let sender = sender.clone();
                    tokio::spawn(async move {
                        let sequence = message.sequence;

                        let result = txm.sync_offchain_message(pool_id, message).await;
                        if let Err(err) = result {
                            error!("[{}] sync offchain message completed with error. {}", sender, err);
                        }

                        bus.send_messages_event(
                            MessagesEvent::Completed((sender.clone(), sequence))
                        );
                    });
                }
            },
            MessagesEvent::Completed((sender, sequence)) => {
                let sender_context = match sender_contexts.get_mut(&sender) {
                    Some(ctx) => ctx,
                    None => {
                        error!("[{}] sender does not found", sender);
                        continue;
                    },
                };
                let _message_context = match sender_context.pending_messages.remove(&sequence) {
                    Some(_) => {},
                    None => {
                        error!("[{}] sequence {} does not found, cannot remove", sender, sequence);
                        continue;
                    },
                };
            },
        }
    }

    Ok(())
}