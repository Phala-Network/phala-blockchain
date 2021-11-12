use anyhow::{anyhow, Result};
use log::{error, info};
use sp_runtime::generic::Era;
use std::time::Duration;

use crate::{
    chain_client::{mq_next_sequence, update_signer_nonce},
    types::{Hash, ParachainApi, PrClient, Signer, SrSigner},
};
use phaxt::extra::{EraInfo, ExtraConfig};

// TODO.kevin: This struct is no longer needed. Just use a simple function to do the job.
/// Hold everything needed to sync some egress messages back to the blockchain
pub struct MsgSync<'a> {
    /// Subxt client
    api: &'a ParachainApi,
    /// pRuntime client
    pr: &'a PrClient,
    /// SR25519 signer with a nonce
    signer: &'a mut SrSigner,
    /// True if the nonce is ever updated from the blockchain during the lifetiem of MsgSync
    nonce_updated: bool,
    /// Extra transcation fee
    tip: u64,
    /// The transection longevity
    longevity: u64,
    /// Max number of messages to sync at a time.
    max_sync_msgs_per_round: u64,
}

impl<'a> MsgSync<'a> {
    /// Creates a new MsgSync object
    pub fn new(
        api: &'a ParachainApi,
        pr: &'a PrClient,
        signer: &'a mut SrSigner,
        tip: u64,
        longevity: u64,
        max_sync_msgs_per_round: u64,
    ) -> Self {
        Self {
            api,
            pr,
            signer,
            nonce_updated: false,
            tip,
            longevity,
            max_sync_msgs_per_round,
        }
    }

    pub async fn maybe_sync_mq_egress(&mut self) -> Result<()> {
        // Send the query
        let messages = self.pr.get_egress_messages(()).await?.decode_messages()?;

        // No pending message. We are done.
        if messages.is_empty() {
            return Ok(());
        }

        self.maybe_update_signer_nonce().await?;

        let era = if self.longevity > 0 {
            let header = self
                .api
                .client
                .rpc()
                .header(<Option<Hash>>::None)
                .await?
                .ok_or_else(|| anyhow!("No header"))?;
            let number = header.number as u64;
            let period = self.longevity;
            let phase = number % period;
            let era = Era::Mortal(period, phase);
            info!(
                "update era: block={}, period={}, phase={}, birth={}, death={}",
                number,
                period,
                phase,
                era.birth(number),
                era.death(number)
            );
            Some(EraInfo {
                period,
                phase,
                birth_hash: header.hash(),
            })
        } else {
            None
        };

        let mut sync_msgs_count = 0;

        'sync_outer: for (sender, messages) in messages {
            if messages.is_empty() {
                continue;
            }
            let min_seq = mq_next_sequence(self.api, &sender).await?;

            info!("Next seq for {} is {}", sender, min_seq);

            for message in messages {
                if message.sequence < min_seq {
                    info!("{} has been submitted. Skipping...", message.sequence);
                    continue;
                }
                let msg_info = format!(
                    "sender={} seq={} dest={} nonce={:?}",
                    sender,
                    message.sequence,
                    String::from_utf8_lossy(&message.message.destination.path()[..]),
                    self.signer.nonce()
                );
                info!("Submitting message: {}", msg_info);
                let extrinsic = self
                    .api
                    .tx()
                    .phala_mq()
                    .sync_offchain_message(message.into())
                    .create_signed(
                        self.signer,
                        ExtraConfig {
                            tip: self.tip,
                            era: era.clone(),
                        },
                    )
                    .await;
                self.signer.increment_nonce();
                match extrinsic {
                    Ok(extrinsic) => {
                        let api = ParachainApi::from(self.api.client.clone());
                        tokio::spawn(async move {
                            const TIMEOUT: u64 = 120;
                            let fut = api.client.rpc().submit_extrinsic(extrinsic);
                            let result =
                                tokio::time::timeout(Duration::from_secs(TIMEOUT), fut).await;
                            match result {
                                Err(_) => {
                                    error!("Submit message timed out: {}", msg_info);
                                }
                                Ok(Err(err)) => {
                                    error!("Error submitting message {}: {:?}", msg_info, err);
                                }
                                Ok(Ok(hash)) => {
                                    info!("Message submited: {} xt-hash={:?}", msg_info, hash);
                                }
                            }
                        });
                    }
                    Err(err) => {
                        panic!("Failed to sign the call: {:?}", err);
                    }
                }
                sync_msgs_count += 1;
                if sync_msgs_count >= self.max_sync_msgs_per_round {
                    info!("Synced {} messages, take a break", sync_msgs_count);
                    break 'sync_outer;
                }
            }
        }
        Ok(())
    }

    /// Updates the nonce if it's not updated.
    ///
    /// The nonce will only be updated once during the lifetime of MsgSync struct.
    async fn maybe_update_signer_nonce(&mut self) -> Result<()> {
        if !self.nonce_updated {
            update_signer_nonce(self.api, &mut *self.signer).await?;
            self.nonce_updated = true;
        }
        Ok(())
    }
}
