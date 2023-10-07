use hex_fmt::HexFmt;
use log::info;
use scale::Encode;
use sideabi::Request;
use sidevm::{
    channel::incoming_queries,
    local_contract,
    logger::{LevelFilter, Logger},
    ocall,
};

#[derive(Debug, Encode)]
pub enum Query {
    InkMessage {
        payload: Vec<u8>,
        /// Amount of tokens deposit to the caller.
        deposit: u128,
        /// Amount of tokens transfer from the caller to the target contract.
        transfer: u128,
        /// Whether to use the gas estimation mode.
        estimating: bool,
    },
    SidevmQuery(Vec<u8>),
}

#[sidevm::main]
async fn main() {
    Logger::with_max_level(LevelFilter::Debug).init();
    let vmid = ocall::vmid().expect("failed to get vmid");

    info!("Test program started! vmid={}", HexFmt(&vmid));

    while let Some(query) = incoming_queries().next().await {
        let request: Request =
            serde_json::from_slice(&query.payload).expect("failed to parse query");
        info!("Received query {request:?} from {:?}", query.origin);
        match request {
            Request::Ping => query.reply_tx.send(b"pong").expect("failed to send reply"),
            Request::Callback { call_data } => {
                let reply = local_contract::query_pink(vmid, call_data)
                    .await
                    .expect("query failed");
                info!("Sending query result: {:?}", reply);
                query.reply_tx.send(&reply).expect("failed to send reply");
            }
        }
    }
}
