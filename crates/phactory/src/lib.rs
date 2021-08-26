#![warn(unused_imports)]
#![warn(unused_extern_crates)]
#![feature(bench_black_box)]
#![feature(panic_unwind)]
#![feature(c_variadic)]

extern crate runtime as chain;

use std;

use rand::*;

use crate::light_validation::LightValidation;
use crate::secret_channel::PeelingReceiver;
use std::collections::BTreeMap;
use std::str;

use anyhow::Result;
use core::convert::TryInto;
use log::{debug, error, info, warn};
use parity_scale_codec::{Decode, Encode};
use ring::rand::SecureRandom;
use serde::{de, Deserialize, Serialize};
use serde_json::{Map, Value};
use sp_core::{crypto::Pair, sr25519, H256};

// use pink::InkModule;

use phactory_api::prpc::InitRuntimeResponse;
use phactory_api::storage_sync::{
    ParachainSynchronizer, SolochainSynchronizer, StorageSynchronizer,
};
use phactory_api::{
    blocks::{self, SyncCombinedHeadersReq, SyncParachainHeaderReq},
};

use phala_crypto::{
    aead,
    ecdh::EcdhKey,
    sr25519::{Persistence, Sr25519SecretKey, KDF, SEED_BYTES},
};
use phala_mq::{BindTopic, ContractId, MessageDispatcher, MessageOrigin, MessageSendQueue};
use phala_pallets::pallet_mq;
use phala_types::WorkerRegistrationInfo;

mod pal;
mod benchmark;
mod contracts;
mod cryptography;
mod light_validation;
mod prpc_service;
mod rpc_types;
mod secret_channel;
mod storage;
mod system;
mod types;
mod utils;

use crate::light_validation::utils::storage_map_prefix_twox_64_concat;
use contracts::{ExecuteEnv, SYSTEM};
use rpc_types::*;
use storage::{Storage, StorageExt};
use types::BlockInfo;
use types::Error;

// TODO: Completely remove the reference to Phala/Khala runtime. Instead we can create a minimal
// runtime definition locally.
type RuntimeHasher = <chain::Runtime as frame_system::Config>::Hashing;

struct RuntimeState {
    contracts: BTreeMap<ContractId, Box<dyn contracts::Contract + Send>>,
    send_mq: MessageSendQueue,
    recv_mq: MessageDispatcher,

    // chain storage synchonizing
    storage_synchronizer: Box<dyn StorageSynchronizer + Send>,
    chain_storage: Storage,
    genesis_block_hash: H256,
    identity_key: sr25519::Pair,
    ecdh_key: EcdhKey,
}

impl RuntimeState {
    fn purge_mq(&mut self) {
        self.send_mq.purge(|sender| {
            use pallet_mq::StorageMapTrait as _;
            type OffchainIngress = pallet_mq::OffchainIngress<chain::Runtime>;

            let module_prefix = OffchainIngress::module_prefix();
            let storage_prefix = OffchainIngress::storage_prefix();
            let key = storage_map_prefix_twox_64_concat(module_prefix, storage_prefix, sender);
            let sequence: u64 = self.chain_storage.get_decoded(&key).unwrap_or(0);
            debug!("purging, sequence = {}", sequence);
            sequence
        })
    }
}

struct FactoryConfig<Platform> {
    platform: Platform,
    machine_id: Vec<u8>,
    dev_mode: bool,
    skip_ra: bool,
    sealing_path: String,
}

struct Phactory<Platform> {
    config: FactoryConfig<Platform>,
    runtime_info: Option<InitRuntimeResponse>,
    runtime_state: Option<RuntimeState>,
}

impl<Platform> Phactory<Platform> {
    pub fn new(platform: Platform) -> Self {
        Phactory {
            config: FactoryConfig {
                platform,
                machine_id: Default::default(), // TODO.kevin: get machine id from platform?
                dev_mode: false,
                skip_ra: false,
                sealing_path: Default::default(),
            },
            runtime_info: None,
            runtime_state: None,
        }
    }
}

const SEAL_DATA_BUF_MAX_LEN: usize = 2048_usize;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
struct PersistentRuntimeData {
    version: u32,
    genesis_block_hash: String,
    sk: String,
    dev_mode: bool,
}

fn save_secret_keys(
    genesis_block_hash: H256,
    sr25519_sk: sr25519::Pair,
    dev_mode: bool,
) -> Result<PersistentRuntimeData> {
    // Put in PresistentRuntimeData
    let serialized_sk = sr25519_sk.dump_secret_key();

    let data = PersistentRuntimeData {
        version: 1,
        genesis_block_hash: hex::encode(genesis_block_hash.as_ref()),
        sk: hex::encode(&serialized_sk),
        dev_mode,
    };
    let encoded_vec = serde_cbor::to_vec(&data).unwrap();
    let encoded_slice = encoded_vec.as_slice();
    info!("Length of encoded slice: {}", encoded_slice.len());

    // Seal
    let aad: [u8; 0] = [0_u8; 0];
    let sealed_data =
        SgxSealedData::<[u8]>::seal_data(&aad, encoded_slice).map_err(anyhow::Error::msg)?;

    let mut return_output_buf = vec![0; SEAL_DATA_BUF_MAX_LEN].into_boxed_slice();
    let output_len: usize = return_output_buf.len();

    let output_slice = &mut return_output_buf;
    let output_ptr = output_slice.as_mut_ptr();

    let opt = to_sealed_log_for_slice(&sealed_data, output_ptr, output_len as u32);
    if opt.is_none() {
        return Err(anyhow::Error::msg(
            sgx_status_t::SGX_ERROR_INVALID_PARAMETER,
        ));
    }

    // TODO: check retval and result
    let mut _retval = sgx_status_t::SGX_SUCCESS;
    let _result = unsafe { ocall_save_persistent_data(&mut _retval, output_ptr, output_len) };
    info!("Persistent Runtime Data saved");
    Ok(data)
}

fn load_secret_keys() -> Result<PersistentRuntimeData> {
    // Try load persisted sealed data
    let mut sealed_data_buf = vec![0; SEAL_DATA_BUF_MAX_LEN].into_boxed_slice();
    let mut sealed_data_len: usize = 0;
    let sealed_data_slice = &mut sealed_data_buf;
    let sealed_data_ptr = sealed_data_slice.as_mut_ptr();
    let sealed_data_len_ptr = &mut sealed_data_len as *mut usize;

    let mut retval = sgx_status_t::SGX_SUCCESS;
    let load_result = unsafe {
        ocall_load_persistent_data(
            &mut retval,
            sealed_data_ptr,
            sealed_data_len_ptr,
            SEAL_DATA_BUF_MAX_LEN,
        )
    };
    if load_result != sgx_status_t::SGX_SUCCESS || sealed_data_len == 0 {
        return Err(anyhow::Error::msg(Error::PersistentRuntimeNotFound));
    }

    let opt = from_sealed_log_for_slice::<u8>(sealed_data_ptr, sealed_data_len as u32);
    let sealed_data = match opt {
        Some(x) => x,
        None => {
            panic!("Sealed data corrupted or outdated, please delete it.")
        }
    };

    let unsealed_data = sealed_data.unseal_data().map_err(anyhow::Error::msg)?;
    let encoded_slice = unsealed_data.get_decrypt_txt();
    info!("Length of encoded slice: {}", encoded_slice.len());

    serde_cbor::from_slice(encoded_slice).map_err(|_| anyhow::Error::msg(Error::DecodeError))
}

fn new_sr25519_key() -> sr25519::Pair {
    let mut rng = rand::thread_rng();
    let mut seed = [0_u8; SEED_BYTES];
    rng.fill_bytes(&mut seed);
    sr25519::Pair::from_seed(&seed)
}

// TODO.kevin: Move to enclave-api when the std ready.
fn generate_random_iv() -> aead::IV {
    let mut nonce_vec = [0u8; aead::IV_BYTES];
    let rand = ring::rand::SystemRandom::new();
    rand.fill(&mut nonce_vec).unwrap();
    nonce_vec
}

fn generate_random_info() -> [u8; 32] {
    let mut nonce_vec = [0u8; 32];
    let rand = ring::rand::SystemRandom::new();
    rand.fill(&mut nonce_vec).unwrap();
    nonce_vec
}

fn init_secret_keys(
    local_state: &mut Phactory,
    genesis_block_hash: H256,
    predefined_identity_key: Option<sr25519::Pair>,
) -> Result<PersistentRuntimeData> {
    let data = if let Some(sr25519_sk) = predefined_identity_key {
        save_secret_keys(genesis_block_hash, sr25519_sk, true)?
    } else {
        match load_secret_keys() {
            Ok(data) => data,
            Err(e)
                if e.is::<Error>()
                    && matches!(
                        e.downcast_ref::<Error>().unwrap(),
                        Error::PersistentRuntimeNotFound
                    ) =>
            {
                warn!("Persistent data not found.");
                let sr25519_sk = new_sr25519_key();
                save_secret_keys(genesis_block_hash, sr25519_sk, false)?
            }
            other_err => return other_err,
        }
    };

    // check genesis block hash
    let saved_genesis_block_hash: [u8; 32] = hex::decode(&data.genesis_block_hash)
        .expect("Unable to decode genesis block hash hex")
        .as_slice()
        .try_into()
        .expect("slice with incorrect length");
    let saved_genesis_block_hash = H256::from(saved_genesis_block_hash);
    if genesis_block_hash != saved_genesis_block_hash {
        panic!(
            "Genesis block hash mismatches with saved keys, expected {}",
            saved_genesis_block_hash
        );
    }

    // load identity
    let sr25519_raw_key: Sr25519SecretKey = hex::decode(&data.sk)
        .expect("Unable to decode identity key hex")
        .as_slice()
        .try_into()
        .expect("slice with incorrect length");

    let sr25519_sk = sr25519::Pair::restore_from_secret_key(&sr25519_raw_key);
    info!("Identity pubkey: {:?}", hex::encode(&sr25519_sk.public()));

    // derive ecdh key
    let ecdh_key = sr25519_sk
        .derive_ecdh_key()
        .expect("Unable to derive ecdh key");
    let ecdh_hex_pk = hex::encode(ecdh_key.public().as_ref());
    info!("ECDH pubkey: {:?}", ecdh_hex_pk);

    // Generate Seal Key as Machine Id
    // This SHOULD be stable on the same CPU
    let machine_id = generate_seal_key();
    info!("Machine id: {:?}", hex::encode(&machine_id));

    // Save
    local_state.genesis_block_hash = Some(genesis_block_hash);
    local_state.identity_key = Some(sr25519_sk);
    local_state.ecdh_key = Some(ecdh_key);
    local_state.machine_id = machine_id;
    local_state.dev_mode = data.dev_mode;

    info!("Init done.");
    Ok(data)
}

// --------------------------------

fn display(e: impl core::fmt::Display) -> Value {
    error_msg(&e.to_string())
}

fn error_msg(msg: &str) -> Value {
    json!({ "message": msg })
}

fn unknown() -> Result<Value, Value> {
    Err(json!({
        "message": "Unknown action"
    }))
}

fn init_runtime(input: InitRuntimeReq) -> Result<Value, Value> {
    // load chain genesis
    let raw_genesis =
        base64::decode(&input.bridge_genesis_info_b64).expect("Bad bridge_genesis_info_b64");
    let genesis =
        light_validation::BridgeInitInfo::<chain::Runtime>::decode(&mut raw_genesis.as_slice())
            .expect("Can't decode bridge_genesis_info_b64");

    // load identity
    let debug_set_key = if let Some(key) = input.debug_set_key {
        Some(hex::decode(&key).map_err(|_| error_msg("Can't decode key hex"))?)
    } else {
        None
    };

    let operator = match input.operator_hex {
        Some(h) => {
            let raw_address =
                hex::decode(h).map_err(|_| error_msg("Error decoding operator_hex"))?;
            Some(chain::AccountId::new(
                raw_address
                    .try_into()
                    .map_err(|_| error_msg("Bad operator_hex"))?,
            ))
        }
        None => None,
    };

    let genesis_state_scl = base64::decode(input.genesis_state_b64)
        .map_err(|_| error_msg("Base64 decode genesis state failed"))?;
    let mut genesis_state_scl = &genesis_state_scl[..];
    let genesis_state: Vec<(Vec<u8>, Vec<u8>)> = Decode::decode(&mut genesis_state_scl)
        .map_err(|_| error_msg("Scale decode genesis state failed"))?;

    let genesis = blocks::GenesisBlockInfo {
        block_header: genesis.block_header,
        validator_set: genesis.validator_set,
        validator_set_proof: genesis.validator_set_proof,
    };

    let resp = prpc_service::init_runtime(
        input.skip_ra,
        input.is_parachain,
        genesis,
        genesis_state,
        operator,
        debug_set_key,
    )
    .map_err(display)?;
    let resp = convert_runtime_info(resp)?;
    Ok(serde_json::to_value(resp).unwrap())
}

fn sync_header(input: blocks::SyncHeaderReq) -> Result<Value, Value> {
    let resp =
        prpc_service::sync_header(input.headers, input.authority_set_change).map_err(display)?;
    Ok(json!({ "synced_to": resp.synced_to }))
}

fn sync_para_header(input: SyncParachainHeaderReq) -> Result<Value, Value> {
    let resp = prpc_service::sync_para_header(input.headers, input.proof).map_err(display)?;
    Ok(json!({ "synced_to": resp.synced_to }))
}

fn sync_combined_headers(input: SyncCombinedHeadersReq) -> Result<Value, Value> {
    let resp = prpc_service::sync_combined_headers(
        input.relaychain_headers,
        input.authority_set_change,
        input.parachain_headers,
        input.proof,
    )
    .map_err(display)?;
    Ok(json!({
        "relaychain_synced_to": resp.relaychain_synced_to,
        "parachain_synced_to": resp.parachain_synced_to,
    }))
}

fn dispatch_block(input: blocks::DispatchBlockReq) -> Result<Value, Value> {
    let resp = prpc_service::dispatch_block(input.blocks).map_err(display)?;
    Ok(json!({ "dispatched_to": resp.synced_to }))
}

fn handle_inbound_messages(
    block_number: chain::BlockNumber,
    state: &mut RuntimeState,
) -> Result<(), Value> {
    // Dispatch events
    let messages = state
        .chain_storage
        .mq_messages()
        .map_err(|_| error_msg("Can not get mq messages from storage"))?;

    let system = &mut SYSTEM_STATE.lock().unwrap();
    let system = system
        .as_mut()
        .ok_or_else(|| error_msg("Runtime not initialized"))?;

    state.recv_mq.reset_local_index();

    for message in messages {
        use phala_types::messaging::SystemEvent;
        macro_rules! log_message {
            ($msg: expr, $t: ident) => {{
                let event: Result<$t, _> =
                    parity_scale_codec::Decode::decode(&mut &$msg.payload[..]);
                match event {
                    Ok(event) => {
                        info!(
                            "mq dispatching message: sender={:?} dest={:?} payload={:?}",
                            $msg.sender, $msg.destination, event
                        );
                    }
                    Err(_) => {
                        info!("mq dispatching message (decode failed): {:?}", $msg);
                    }
                }
            }};
        }
        // TODO.kevin: reuse codes in debug-cli
        if message.destination.path() == &SystemEvent::topic() {
            log_message!(message, SystemEvent);
        } else {
            info!("mq dispatching message: {:?}", message);
        }
        state.recv_mq.dispatch(message);
    }

    let mut guard = scopeguard::guard(&mut state.recv_mq, |mq| {
        let n_unhandled = mq.clear();
        if n_unhandled > 0 {
            warn!("There are {} unhandled messages dropped", n_unhandled);
        }
    });

    let now_ms = state
        .chain_storage
        .timestamp_now()
        .ok_or_else(|| error_msg("No timestamp found in block"))?;

    let storage = &state.chain_storage;
    let recv_mq = &mut *guard;
    let mut block = BlockInfo {
        block_number,
        now_ms,
        storage,
        recv_mq,
    };

    if let Err(e) = system.process_messages(&mut block) {
        error!("System process events failed: {:?}", e);
        return Err(error_msg("System process events failed"));
    }

    let mut env = ExecuteEnv {
        block: &block,
        system,
    };

    for contract in state.contracts.values_mut() {
        contract.process_messages(&mut env);
    }

    Ok(())
}

// fn get_info_json() -> Result<Value, Value> {
//     let info = prpc_service::get_info();
//     let machine_id = LOCAL_STATE.lock().unwrap().machine_id;
//     let machine_id = hex::encode(&machine_id);
//     let gatekeeper = info.gatekeeper.unwrap();
//     Ok(json!({
//         "initialized": info.initialized,
//         "registered": info.registered,
//         "gatekeeper": {
//             "role": gatekeeper.role,
//             "master_public_key": gatekeeper.master_public_key,
//         },
//         "genesis_block_hash": info.genesis_block_hash,
//         "public_key": info.public_key,
//         "ecdh_public_key": info.ecdh_public_key,
//         "headernum": info.headernum,
//         "para_headernum": info.para_headernum,
//         "blocknum": info.blocknum,
//         "state_root": info.state_root,
//         "dev_mode": info.dev_mode,
//         "pending_messages": info.pending_messages,
//         "score": info.score,
//         "machine_id": machine_id,
//     }))
// }
