use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use log::{debug, error, info, trace};

use phala_types::messaging::{MessageOrigin, SignedMessage};
use tokio::sync::mpsc;

use crate::{bus::Bus, tx::TxManager};

pub enum MessagesEvent {
    SyncMessages((u64, MessageOrigin, Vec<SignedMessage>)),
    Completed((MessageOrigin, u64, bool)),
}

pub type MessagesRx = mpsc::UnboundedReceiver<MessagesEvent>;
pub type MessagesTx = mpsc::UnboundedSender<MessagesEvent>;

pub enum MessageState {
    Trying,
    Successful,
    Failure,
}

pub struct MessageContext {
    state: MessageState,
    prev_try_count: usize,
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

                    match sender_context.pending_messages.entry(message.sequence) {
                        std::collections::hash_map::Entry::Occupied(entry) => {
                            let message_context = entry.into_mut();
                            match message_context.state {
                                MessageState::Trying => {
                                    trace!("[{}] message #{} is in progress.", sender, message.sequence);
                                    continue;
                                },
                                MessageState::Successful => {
                                    trace!("[{}] message #{} was successful.", sender, message.sequence);
                                    continue;
                                },
                                MessageState::Failure => {
                                    message_context.state = MessageState::Trying;
                                    info!(
                                        "[{}] message #{} was failed for {} times. Trying again now..",
                                        sender, message.sequence, message_context.prev_try_count
                                    );
                                    message_context.prev_try_count += 1;
                                },
                            }
                        },
                        std::collections::hash_map::Entry::Vacant(entry) => {
                            entry.insert(MessageContext {
                                state: MessageState::Trying,
                                prev_try_count: 0,
                            });
                        },
                    }

                    let bus = bus.clone();
                    let txm = txm.clone();
                    let sender = sender.clone();
                    tokio::spawn(async move {
                        let sequence = message.sequence;

                        let result = txm.sync_offchain_message(pool_id, message).await;
                        if let Err(err) = &result {
                            error!("[{}] sync offchain message completed with error. {}", sender, err);
                        }

                        let _ = bus.send_messages_event(
                            MessagesEvent::Completed((sender.clone(), sequence, result.is_ok()))
                        );
                    });
                }
            },
            MessagesEvent::Completed((sender, sequence, ok)) => {
                let sender_context = match sender_contexts.get_mut(&sender) {
                    Some(ctx) => ctx,
                    None => {
                        error!("[{}] sender does not found", sender);
                        continue;
                    },
                };
                match sender_context.pending_messages.get_mut(&sequence) {
                    Some(ctx) => {
                        ctx.state = if ok {
                            MessageState::Successful
                        } else {
                            MessageState::Failure
                        };
                    },
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