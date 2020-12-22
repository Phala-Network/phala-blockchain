use std::cmp;
use codec::{Encode, Decode};
use core::marker::PhantomData;

use super::{
    update_signer_nonce,
    error::Error,
    types::{ReqData, QueryRespData, TransferData, TxQueue},
    runtimes,
    XtClient, PrClient, SrSigner,
    BALANCES, ASSETS
};

/// Hold everything needed to sync some egress messages back to the blockchain
pub struct MsgSync<'a> {
    /// Subxt client
    client: &'a XtClient,
    /// pRuntime client
    pr: &'a PrClient,
    /// SR25519 signer with a nonce
    signer: &'a mut SrSigner,
    /// True if the nonce is ever updated from the blockchain during the lifetiem of MsgSync
    nonce_updated: bool,
}

impl<'a> MsgSync<'a> {
    /// Creates a new MsgSync object
    pub fn new(client: &'a XtClient, pr: &'a PrClient, signer: &'a mut SrSigner) -> Self {
        Self {
            client, pr, signer,
            nonce_updated: false,
        }
    }

    /// Syncs the worker egress messages when available
    pub async fn maybe_sync_worker_egress(&mut self, sequence: &mut u64) -> Result<(), Error> {
        // Check pending messages in worker egress queue
        let query_resp = self.pr.query(
            0, ReqData::GetWorkerEgress { start_sequence: *sequence }).await?;
        let msg_data = match query_resp {
            QueryRespData::GetWorkerEgress { length, encoded_egreee_b64 } => {
                println!("maybe_sync_worker_egress: got {} messages", length);
                base64::decode(&encoded_egreee_b64)
                    .map_err(|_| Error::FailedToDecode)?
            }
            _ => return Err(Error::FailedToDecode)
        };
        let msg_queue: Vec<phala_types::SignedWorkerMessage> = Decode::decode(&mut &msg_data[..])
            .map_err(|_| Error::FailedToDecode)?;
        // No pending message. We are done.
        if msg_queue.len() == 0 {
            return Ok(());
        }
        // Send messages
        self.maybe_update_signer_nonce().await?;
        let mut next_seq = *sequence;
        for msg in &msg_queue {
            let msg_seq = msg.data.sequence;
            if msg_seq < *sequence {    // This seq is 0-based
                println!("Worker msg {} has been submitted. Skipping...", msg_seq);
                continue;
            }
            // STATUS: claim_tx_sent = match msg.data.payload { WorkerMessagePayload::Heartbeat { block_num, ... } => block_num }
            next_seq = cmp::max(next_seq, msg_seq + 1);
            let ret = self.client.submit(runtimes::phala::SyncWorkerMessageCall {
                _runtime: PhantomData,
                msg: msg.encode(),
            }, self.signer).await;
            if let Err(err) = ret {
                println!("Failed to submit tx: {:?}", err);
                // TODO: Should we fail early?
                // STATUS: worth reporting error!
            }
            self.signer.increment_nonce();
        }
        *sequence = next_seq;
        Ok(())
    }

    /// Syncs the Balances egress messages when available
    pub async fn maybe_sync_contract_egress(&mut self, sequence: &mut u64, contract_id: u32) -> Result<(), Error> {
        // Check pending messages in Balances' egress queue
        let query_resp = self.pr.query(contract_id, ReqData::PendingChainTransfer { sequence: *sequence }).await?;
        let transfer_data = match query_resp {
            QueryRespData::PendingChainTransfer { transfer_queue_b64 } =>
                base64::decode(&transfer_queue_b64)
                    .map_err(|_| Error::FailedToDecode)?,
            _ => return Err(Error::FailedToDecode)
        };
        if contract_id == BALANCES {
            self.submit_balance_transactions(sequence, transfer_data).await
        } else {
            self.submit_asset_transactions(sequence, transfer_data).await
        }
    }

    async fn submit_balance_transactions(&mut self, sequence: &mut u64, transfer_data: Vec<u8>) -> Result<(), Error> {
        let transfer_queue: Vec<TransferData> = Decode::decode(&mut &transfer_data[..])
            .map_err(|_|Error::FailedToDecode)?;
        // No pending message. We are done.
        if transfer_queue.len() == 0 {
            return Ok(());
        }
        // Send messages
        self.maybe_update_signer_nonce().await?;
        let mut next_seq = *sequence;
        for transfer_data in &transfer_queue {
            let msg_seq = transfer_data.data.sequence;
            if msg_seq <= *sequence {   // This seq is 1-based
                println!("Msg {} has been submitted. Skipping...", msg_seq);
                continue;
            }
            next_seq = cmp::max(next_seq, msg_seq);
            let ret = self.client.submit(runtimes::phala::TransferToChainCall {
                _runtime: PhantomData,
                data: transfer_data.encode()
            }, self.signer).await;
            if let Err(err) = ret {
                println!("Failed to submit tx: {:?}", err);
                // TODO: Should we fail early?
            }
            self.signer.increment_nonce();
        }
        *sequence = next_seq;
        Ok(())
    }

    async fn submit_asset_transactions(&mut self, sequence: &mut u64, transfer_data: Vec<u8>) -> Result<(), Error> {
        let transfer_queue: Vec<TxQueue> = Decode::decode(&mut &transfer_data[..])
            .map_err(|_|Error::FailedToDecode)?;
        if transfer_queue.len() == 0 {
            return Ok(());
        }

        // Send messages
        self.maybe_update_signer_nonce().await?;
        let mut next_seq = *sequence;
        for transfer_data in &transfer_queue {
            match transfer_data {
                TxQueue::TransferTokenData(transfer_data) => {
                    let msg_seq = transfer_data.data.sequence;
                    if msg_seq <= *sequence {   // This seq is 1-based
                        println!("Msg {} has been submitted. Skipping...", msg_seq);
                        continue;
                    }
                    next_seq = cmp::max(next_seq, msg_seq);
                    let ret = self.client.submit(runtimes::phala::TransferTokenToChainCall {
                        _runtime: PhantomData,
                        data: transfer_data.encode()
                    }, self.signer).await;
                    if let Err(err) = ret {
                        println!("Failed to submit tx: {:?}", err);
                        // TODO: Should we fail early?
                    }
                },
                TxQueue::TransferXTokenData(transfer_data) => {
                    let msg_seq = transfer_data.data.sequence;
                    if msg_seq <= *sequence {   // This seq is 1-based
                        println!("Msg {} has been submitted. Skipping...", msg_seq);
                        continue;
                    }
                    next_seq = cmp::max(next_seq, msg_seq);
                    let ret = self.client.submit(runtimes::phala::TransferXTokenToChainCall {
                        _runtime: PhantomData,
                        data: transfer_data.encode()
                    }, self.signer).await;
                    if let Err(err) = ret {
                        println!("Failed to submit tx: {:?}", err);
                        // TODO: Should we fail early?
                    }
                },
            }
            self.signer.increment_nonce();
        }

        *sequence = next_seq;

        Ok(())
    }

    /// Updates the nonce if it's not updated.
    ///
    /// The nonce will only be updated once during the lifetime of MsgSync struct.
    async fn maybe_update_signer_nonce(&mut self) -> Result<(), Error> {
        if !self.nonce_updated {
            update_signer_nonce(self.client, self.signer).await?;
            self.nonce_updated = true;
        }
        Ok(())
    }
}
