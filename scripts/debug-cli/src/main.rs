use codec::Decode;
use structopt::StructOpt;

// use phala_types;

#[derive(Debug, StructOpt)]
#[structopt(name = "Phala Debug Utility CLI")]
enum Cli {
    DecodeWorkerMessage {
        #[structopt(short)]
        hex_data: String,
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
        }
    }
}
