mod query;

use clap::{AppSettings, Parser, Subcommand};
use codec::{Decode, Encode};
use phala_types::contract::ContractId;
use std::convert::TryInto;
use std::fmt::Debug;

// use phala_types;

#[derive(Debug, Parser)]
#[clap(name = "Phala Debug Utility CLI")]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
enum Cli {
    DecodeWorkerRegistrationInfo {
        #[clap(short)]
        hex_data: String,
        #[clap(long)]
        print_field: Option<String>,
    },
    DecodeRaQuote {
        #[clap(short)]
        b64_data: String,
    },
    DecodeHeader {
        #[clap(short)]
        hex_data: String,
    },
    DecodeBhwe {
        #[clap(short)]
        b64_data: String,
    },
    DecodeEgressMessages {
        b64_data: String,
    },
    DecodeSignedMessage {
        #[clap(short)]
        hex_data: String,
    },
    DecodeBridgeLotteryMessage {
        #[clap(short)]
        hex_data: String,
    },
    DecodeMqPayload {
        destination: String,
        hex_data: String,
    },
    DecodeFrnkJustification {
        hex_data: String,
    },
    DecodeGenesisBlockInfo {
        b64_data: String,
    },
    EcdhKey {
        privkey: String,
    },
    InspectPalletId {
        pallet_id: String,
    },
    Rpc {
        #[clap(long, default_value = "http://localhost:8000")]
        url: String,

        #[clap(subcommand)]
        command: RpcCommand,
    },
    Pink {
        #[clap(subcommand)]
        command: PinkCommand,
    },
}

#[derive(Debug, Subcommand)]
enum RpcCommand {
    GetWorkerState { pubkey: String },
    GetInfo,
}

#[derive(Debug, Subcommand)]
enum PinkCommand {
    Query {
        #[clap(long, default_value = "http://localhost:8000")]
        url: String,
        #[clap(long)]
        sidevm: bool,
        id: String,
        message: String,
    },
    Command {
        id: String,
        message: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli {
        Cli::DecodeWorkerRegistrationInfo {
            hex_data,
            print_field,
        } => {
            let data = decode_hex(&hex_data);
            let msg: phala_types::WorkerRegistrationInfo<sp_runtime::AccountId32> =
                Decode::decode(&mut data.as_slice()).expect("Failed to decode message");
            match print_field {
                Some(f) if f == "machine_id" => println!("{}", hex::encode(&msg.machine_id)),
                _ => println!("Decoded: {:?}", msg),
            }
        }
        Cli::DecodeRaQuote { b64_data } => {
            let quote_body = base64::decode(&b64_data).expect("Failed to decode b64_data");
            let mr_enclave = &quote_body[112..144];
            let mr_signer = &quote_body[176..208];
            let isv_prod_id = &quote_body[304..306];
            let isv_svn = &quote_body[306..308];
            println!("- mr_enclave: {}", hex::encode(&mr_enclave));
            println!("- mr_signer: {}", hex::encode(&mr_signer));
            println!("- isv_prod_id: {}", hex::encode(&isv_prod_id));
            println!("- isv_svn: {}", hex::encode(&isv_svn));
        }
        Cli::DecodeHeader { hex_data } => {
            use sp_runtime::{generic::Header, traits::BlakeTwo256};
            let data = decode_hex(&hex_data);
            let header = Header::<u128, BlakeTwo256>::decode(&mut data.as_slice())
                .expect("Faield to parse Header");
            let hash = header.hash();
            println!("Decoded: {:?}", header);
            println!("Hash: 0x{}", hex::encode(&hash));
        }
        Cli::DecodeBhwe { b64_data } => {
            let data = base64::decode(&b64_data).expect("Failed to decode b64_data");
            let snapshot =
                phactory_api::blocks::BlockHeaderWithChanges::decode(&mut data.as_slice());

            println!("Decoded: {:?}", snapshot);
        }
        Cli::DecodeEgressMessages { b64_data } => {
            let data = decode_b64(&argument_or_stdin(&b64_data));
            let messages = phactory_api::prpc::EgressMessages::decode(&mut data.as_slice());
            println!("Decoded: {:?}", messages);
        }
        Cli::DecodeSignedMessage { hex_data } => {
            use phala_types::messaging::SignedMessage;
            decode_hex_print::<SignedMessage>(&hex_data);
        }
        Cli::DecodeBridgeLotteryMessage { hex_data } => {
            use phala_types::messaging::Lottery;
            let lottery = decode_hex_print::<Lottery>(&hex_data);

            match lottery {
                Lottery::BtcAddresses { address_set } => {
                    let addrs: Vec<_> = address_set
                        .iter()
                        .map(|raw_addr| std::str::from_utf8(&raw_addr).unwrap())
                        .collect();
                    println!("Lottery::BtcAddresses {:?}", addrs);
                }
                _ => {}
            }
        }
        Cli::DecodeMqPayload {
            destination,
            hex_data,
        } => {
            let data = decode_hex(&hex_data);
            decode_mq_payload(destination.as_bytes(), &data);
        }
        Cli::DecodeFrnkJustification { hex_data } => {
            let data = decode_hex(&hex_data);
            let j = sc_finality_grandpa::GrandpaJustification::<Block>::decode(&mut &data[..])
                .expect("Error decoding FRNK justification");
            println!("{:?}", j);
        }
        Cli::DecodeGenesisBlockInfo { b64_data } => {
            let data = decode_b64(&b64_data);
            let gi = phactory_api::blocks::GenesisBlockInfo::decode(&mut &data[..]).unwrap();
            println!("{:?}", gi);
        }
        Cli::EcdhKey { privkey } => {
            use phala_crypto::ecdh;

            let privkey = hex::decode(privkey).expect("Failed to decode hex key");
            let pair =
                ecdh::EcdhKey::create(privkey.as_slice().try_into().expect("Invalid key length"))
                    .unwrap();
            println!("Pubkey: {}", hex::encode(pair.public()));
            println!("Privkey: {}", hex::encode(pair.secret()));
        }
        Cli::InspectPalletId { pallet_id } => {
            use sp_runtime::traits::AccountIdConversion;
            let pallet_id_array: [u8; 8] = pallet_id.as_bytes().try_into().expect("Bad length");
            let id = frame_support::PalletId(pallet_id_array);
            let account: AccountId = id.into_account_truncating();
            println!("Pallet account: {}", account);
        }
        Cli::Rpc { url, command } => {
            handle_rpc_command(command, url).await;
        }
        Cli::Pink { command } => {
            handle_pink_command(command).await;
        }
    }
}

async fn handle_rpc_command(command: RpcCommand, url: String) {
    let client = phactory_api::pruntime_client::new_pruntime_client(url);
    fn print_result<T: Debug, E: Debug>(result: Result<T, E>) {
        match result {
            Ok(result) => println!("{:#?}", result),
            Err(err) => println!("Error: {:?}", err),
        }
    }
    match command {
        RpcCommand::GetWorkerState { pubkey } => {
            let public_key = try_decode_hex(&pubkey).expect("Failed to decode pubkey");
            let rv = client
                .get_worker_state(phactory_api::prpc::GetWorkerStateRequest { public_key })
                .await;
            print_result(rv);
        }
        RpcCommand::GetInfo => {
            let rv = client.get_info(()).await;
            print_result(rv);
        }
    }
}

async fn handle_pink_command(command: PinkCommand) {
    #[derive(Debug, Encode, Decode)]
    pub enum Query {
        InkMessage(Vec<u8>),
        SidevmQuery(Vec<u8>),
    }

    #[derive(Debug, Encode, Decode)]
    pub enum Command {
        InkMessage { nonce: Vec<u8>, message: Vec<u8> },
    }

    #[derive(Debug, Encode, Decode)]
    pub enum Response {
        Payload(Vec<u8>),
    }

    #[derive(Debug, Encode, Decode)]
    pub enum QueryError {
        BadOrigin,
        RuntimeError(String),
        SidevmNotFound,
    }

    match command {
        PinkCommand::Query {
            url,
            sidevm,
            id,
            message,
        } => {
            let id = decode_hex(&id);
            let id = ContractId::decode(&mut &id[..]).expect("Bad contract id");
            let message = decode_hex(&message);
            let query = if sidevm {
                Query::SidevmQuery(message)
            } else {
                Query::InkMessage(message)
            };

            let result: Result<Response, QueryError> =
                query::query(url, id, query).await.expect("Query failed");

            let response = match result {
                Err(err) => {
                    println!("Error: {:?}", err);
                    return;
                }
                Ok(response) => response,
            };
            match response {
                Response::Payload(response) => {
                    let s = std::str::from_utf8(&response).unwrap_or_default();
                    println!("response: [{}]({}))", hex::encode(&response), s);
                }
            }
        }
        PinkCommand::Command { id, message } => {
            #[derive(Encode)]
            enum Payload<T> {
                Plain(T),
            }

            let id = decode_hex(&id);
            let id = ContractId::decode(&mut &id[..]).expect("Bad contract id");
            let message = decode_hex(&message);
            let nonce = vec![];
            let command = Command::InkMessage { nonce, message };
            let mq_payload = Payload::Plain(command);
            println!(
                "topic: (0x{})",
                hex::encode(phala_types::contract::command_topic(id))
            );
            println!("command: (0x{})", hex::encode(mq_payload.encode()));
        }
    }
}

fn try_decode_hex(hex_str: &str) -> Result<Vec<u8>, hex::FromHexError> {
    hex::decode(hex_str.strip_prefix("0x").unwrap_or(hex_str))
}

fn decode_hex(hex_str: &str) -> Vec<u8> {
    try_decode_hex(hex_str).expect("Failed to parse hex_data")
}

fn decode_hex_print<T: Decode + Debug>(hex_data: &str) -> T {
    let data = decode_hex(hex_data);
    let message = T::decode(&mut data.as_slice());
    println!("Decode: {:?}", message);
    message.unwrap()
}

fn decode_b64(b64_str: &str) -> Vec<u8> {
    base64::decode(b64_str.trim()).expect("Failed to decode b64_data")
}

fn argument_or_stdin(arg: &str) -> String {
    use std::io::Read;
    if arg == "-" {
        let mut buffer = String::new();
        let mut stdin = std::io::stdin();
        stdin
            .read_to_string(&mut buffer)
            .expect("Failed to read from stdin");
        buffer
    } else {
        arg.to_string()
    }
}

type AccountId = sp_runtime::AccountId32;
type Header = sp_runtime::generic::Header<u32, sp_runtime::traits::BlakeTwo256>;
type Block = sp_runtime::generic::Block<Header, sp_runtime::OpaqueExtrinsic>;

// TODO(h4x): move it to a separate crate to share the code
fn decode_mq_payload(destination: &[u8], payload: &[u8]) {
    use phala_types::messaging::*;
    fn try_print<T: BindTopic + Decode + std::fmt::Debug>(
        destination: &[u8],
        payload: &[u8],
    ) -> Result<(), ()> {
        if &T::topic() == destination {
            let msg: T = Decode::decode(&mut &payload[..]).expect("Cannot decode message");
            println!("{:#?}", msg);
            return Ok(());
        }
        Err(())
    }
    macro_rules! try_decode_with_types {
        ($($msg: ty,)+) => {
            $(if try_print::<$msg>(destination, payload).is_ok() { return; })+
        }
    }

    try_decode_with_types!(
        //Lottery,
        //LotteryCommand,
        //BalancesEvent<AccountId, Balance>,
        //BalancesCommand<AccountId, Balance>,
        // BalancesTransfer<AccountId, Balance>,
        //AssetCommand<AccountId, Balance>,
        //Web3AnalyticsCommand,
        // KittyEvent<AccountId, Hash>,
        SystemEvent,
        WorkingReportEvent,
        WorkingInfoUpdateEvent<u32>,
        GatekeeperEvent,
        phala_pallets::registry::RegistryEvent,
    );

    println!("Cannot decode.");
}
