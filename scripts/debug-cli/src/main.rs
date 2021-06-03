use codec::Decode;
use std::fmt::Debug;
use structopt::StructOpt;

// use phala_types;

#[derive(Debug, StructOpt)]
#[structopt(name = "Phala Debug Utility CLI")]
enum Cli {
    DecodeWorkerMessage {
        #[structopt(short)]
        hex_data: String,
    },
    DecodeWorkerMessageQueue {
        #[structopt(short)]
        b64_data: String,
    },
    DecodePruntimeInfo {
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
    DecodeWorkerSnapshot {
        #[structopt(short)]
        hex_data: String,
    },
    DecodeBhwe {
        #[structopt(short)]
        b64_data: String,
    },
    DecodeSignedLotteryMessage {
        #[structopt(short)]
        hex_data: String,
    },
    DecodeBridgeLotteryMessage {
        #[structopt(short)]
        hex_data: String,
    },
}

fn main() {
    let cli = Cli::from_args();
    match cli {
        Cli::DecodeWorkerMessage { hex_data } => {
            let data = decode_hex(&hex_data);
            let msg: phala_types::SignedWorkerMessage =
                Decode::decode(&mut data.as_slice()).expect("Failed to decode message");
            println!("Decoded: {:?}", msg);
        }
        Cli::DecodeWorkerMessageQueue { b64_data } => {
            let data = base64::decode(&b64_data).expect("Failed to decode b64_data");
            let msg: Vec<phala_types::SignedWorkerMessage> =
                Decode::decode(&mut data.as_slice()).expect("Failed to decode message");
            println!("Decoded: {:?}", msg);
        }
        Cli::DecodePruntimeInfo {
            hex_data,
            print_field,
        } => {
            let data = decode_hex(&hex_data);
            let msg: phala_types::PRuntimeInfo =
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
        Cli::DecodeWorkerSnapshot { hex_data } => {
            let data = decode_hex(&hex_data);
            let snapshot = phala_types::pruntime::OnlineWorkerSnapshot::<u32, u128>::decode(
                &mut data.as_slice(),
            );

            println!("Decoded: {:?}", snapshot);
        }
        Cli::DecodeBhwe { b64_data } => {
            use sp_runtime::traits::BlakeTwo256;
            let data = base64::decode(&b64_data).expect("Failed to decode b64_data");
            let snapshot =
                phala_types::pruntime::BlockHeaderWithEvents::<u32, BlakeTwo256, u128>::decode(
                    &mut data.as_slice(),
                );

            println!("Decoded: {:?}", snapshot);
        }
        Cli::DecodeSignedLotteryMessage { hex_data } => {
            use phala_types::messaging::SignedLotteryMessage;
            decode_hex_print::<SignedLotteryMessage>(&hex_data);
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
