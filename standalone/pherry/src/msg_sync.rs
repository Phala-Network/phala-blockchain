use anyhow::{anyhow, Result};
use log::{error, info};
use sp_runtime::generic::Era;
use std::time::Duration;

use crate::{
    chain_client::{mq_next_sequence, update_signer_nonce},
    types::{Hash, ParachainApi, PrClient, Signer, SrSigner},
};
use phaxt::extra::{EraInfo, ExtraConfig};

pub async fn maybe_sync_mq_egress(
    api: &ParachainApi,
    pr: &PrClient,
    signer: &mut SrSigner,
    tip: u64,
    longevity: u64,
    max_sync_msgs_per_round: u64,
) -> Result<()> {
    // Send the query
    let messages = pr.get_egress_messages(()).await?.decode_messages()?;

    // No pending message. We are done.
    if messages.is_empty() {
        return Ok(());
    }

    update_signer_nonce(api, signer).await?;

    let era = if longevity > 0 {
        let header = api
            .client
            .rpc()
            .header(<Option<Hash>>::None)
            .await?
            .ok_or_else(|| anyhow!("No header"))?;
        let number = header.number as u64;
        let period = longevity;
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
        let min_seq = mq_next_sequence(api, &sender).await?;

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
                signer.nonce()
            );
            info!("Submitting message: {}", msg_info);
            let extrinsic = api
                .tx()
                .phala_mq()
                .sync_offchain_message(message)
                .create_signed(
                    signer,
                    ExtraConfig {
                        tip,
                        era: era.clone(),
                    },
                )
                .await;
            signer.increment_nonce();
            match extrinsic {
                Ok(extrinsic) => {
                    let api = ParachainApi::from(api.client.clone());
                    tokio::spawn(async move {
                        const TIMEOUT: u64 = 120;
                        let fut = api.client.rpc().submit_extrinsic(extrinsic);
                        let result = tokio::time::timeout(Duration::from_secs(TIMEOUT), fut).await;
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
            if sync_msgs_count >= max_sync_msgs_per_round {
                info!("Synced {} messages, take a break", sync_msgs_count);
                break 'sync_outer;
            }
        }
    }
    Ok(())
}
