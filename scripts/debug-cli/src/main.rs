use codec::Decode;
use structopt::StructOpt;

// use phala_types;

#[derive(Debug, StructOpt)]
#[structopt(name = "Phala Debug Utility CLI")]
enum Cli {
    DecodeWorkerMessage {
        #[structopt(short)]
        hex_data: String,
    },
    DecodeRaQuote {
        #[structopt(short)]
        b64_data: String,
    }
}

fn main() {
    let cli = Cli::from_args();
    match cli {
        Cli::DecodeWorkerMessage { hex_data } => {
            let data = hex::decode(hex_data)
                .expect("Failed to parse hex_data");
             let msg: phala_types::SignedWorkerMessage = Decode::decode(&mut data.as_slice())
                .expect("Failed to decode message");
            println!("Decoded: {:?}", msg);
        },
        Cli::DecodeRaQuote { b64_data } => {
            let quote_body = base64::decode(&b64_data)
                .expect("Failed to decode b64_data");
			let mr_enclave = &quote_body[112..144];
			let mr_signer = &quote_body[176..208];
			let isv_prod_id = &quote_body[304..306];
            let isv_svn = &quote_body[306..308];
            println!("- mr_enclave: {}", hex::encode(&mr_enclave));
            println!("- mr_signer: {}", hex::encode(&mr_signer));
            println!("- isv_prod_id: {}", hex::encode(&isv_prod_id));
            println!("- isv_svn: {}", hex::encode(&isv_svn));
        }
    }
}
