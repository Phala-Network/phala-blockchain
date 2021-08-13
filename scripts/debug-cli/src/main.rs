use codec::{Decode, Encode};
use std::convert::TryInto;
use std::fmt::Debug;
use structopt::StructOpt;

// use phala_types;

#[derive(Debug, StructOpt)]
#[structopt(name = "Phala Debug Utility CLI")]
enum Cli {
    DecodeWorkerRegistrationInfo {
        #[structopt(short)]
        hex_data: String,
        #[structopt(long)]
        print_field: Option<String>,
    },
    DecodeRaQuote {
        #[structopt(short)]
        b64_data: String,
    },
    DecodeHeader {
        #[structopt(short)]
        hex_data: String,
    },
    DecodeBhwe {
        #[structopt(short)]
        b64_data: String,
    },
    DecodeEgressMessages {
        b64_data: String,
    },
    DecodeSignedMessage {
        #[structopt(short)]
        hex_data: String,
    },
    DecodeBridgeLotteryMessage {
        #[structopt(short)]
        hex_data: String,
    },
    DecodeMqPayload {
        destination: String,
        hex_data: String,
    },
    EncodeLotterySetAdmin {
        admin: String,
        number: u64,
    },
    EncodeLotteryUtxo {
        round: u32,
        address: String,
        txid: String,
        p0: u32,
        p1: u64,
    },
    EcdhKey {
        privkey: String,
    },
    InspectPalletId {
        pallet_id: String,
    }
}

fn main() {
    let cli = Cli::from_args();
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
                enclave_api::blocks::BlockHeaderWithChanges::decode(&mut data.as_slice());

            println!("Decoded: {:?}", snapshot);
        }
        Cli::DecodeEgressMessages { b64_data } => {
            let data = decode_b64(&argument_or_stdin(&b64_data));
            let messages = enclave_api::prpc::EgressMessages::decode(&mut data.as_slice());
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
        Cli::EncodeLotterySetAdmin { admin, number } => {
            use phala_types::messaging::{LotteryCommand, PushCommand};
            let payload = PushCommand {
                command: LotteryCommand::SetAdmin { new_admin: admin },
                number,
            };
            println!("payload: 0x{}", hex::encode(payload.encode()));
        }
        Cli::EncodeLotteryUtxo {
            round,
            address,
            txid,
            p0,
            p1,
        } => {
            use phala_types::messaging::{LotteryCommand, PushCommand};
            let mut txid_buf: [u8; 32] = Default::default();
            hex::decode_to_slice(txid, &mut txid_buf).unwrap();
            let payload = PushCommand {
                command: LotteryCommand::SubmitUtxo {
                    round_id: round,
                    address,
                    utxo: (txid_buf, p0, p1),
                },
                number: 1,
            };
            println!("payload: 0x{}", hex::encode(payload.encode()));
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
            let account: AccountId = id.into_account();
            println!("Pallet account: {}", account);
        }
    }
}

fn decode_hex(hex_str: &str) -> Vec<u8> {
    let raw_hex = if hex_str.starts_with("0x") {
        &hex_str[2..]
    } else {
        hex_str
    };
    hex::decode(raw_hex).expect("Failed to parse hex_data")
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
type Balance = u128;

// TODO(h4x): move it to a separate crate to share the code
fn decode_mq_payload(destination: &[u8], payload: &[u8]) {
    use phala_types::messaging::*;
    fn try_print<T: BindTopic + Decode + std::fmt::Debug>(
        destination: &[u8],
        payload: &[u8],
    ) -> Result<(), ()> {
        if &T::topic() == destination {
            let msg: T = Decode::decode(&mut &payload[..]).expect("Cannot decode message");
            println!("{:?}", msg);
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
        //DiemCommand,
        // KittyEvent<AccountId, Hash>,
        SystemEvent,
        MiningReportEvent,
        MiningInfoUpdateEvent<u32>,
        GatekeeperEvent,
        phala_pallets::registry::RegistryEvent,
    );

    println!("Cannot decode.");
}
