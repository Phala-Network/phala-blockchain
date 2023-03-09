use std::str;

use phactory_api::prpc::phactory_api_server::PhactoryAPIMethod;
use rocket::data::{ByteUnit, Data, Limits, ToByteUnit};
use rocket::http::{ContentType, Method, Status};
use rocket::response::status::Custom;
use rocket::serde::json::{json, Json, Value as JsonValue};
use rocket::Phase;
use rocket::{get, post, routes};
use rocket_cors::{AllowedHeaders, AllowedMethods, AllowedOrigins, CorsOptions};

use colored::Colorize as _;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use phactory_api::{actions, prpc};
use phala_rocket_middleware::ResponseSigner;

use crate::runtime;

#[derive(Serialize, Deserialize)]
struct ContractInput {
    input: Map<String, Value>,
    nonce: Map<String, Value>,
}

macro_rules! do_ecall_handle {
    ($num: expr, $content: expr) => {{
        match runtime::ecall_handle($num, $content) {
            Ok(data) => {
                let output_value: serde_json::value::Value = serde_json::from_slice(&data).unwrap();
                json!(output_value)
            },
            Err(err) => {
                error!("Call ecall_handle failed: {:?}!", err);
                json!({
                    "status": "error",
                    "payload": format!("{:?}!", err)
                })
            }
        }
    }};
}

macro_rules! proxy_post {
    ($rpc: literal, $name: ident, $num: expr) => {
        #[post($rpc, format = "json", data = "<contract_input>")]
        fn $name(contract_input: Json<ContractInput>) -> JsonValue {
            debug!(
                "{}",
                ::serde_json::to_string_pretty(&*contract_input).unwrap()
            );

            let input_string = serde_json::to_string(&*contract_input).unwrap();
            do_ecall_handle!($num, input_string.as_bytes())
        }
    };
}

macro_rules! proxy_get {
    ($rpc: literal, $name: ident, $num: expr) => {
        #[get($rpc)]
        fn $name() -> JsonValue {
            let input_string = r#"{ "input": {} }"#.to_string();
            do_ecall_handle!($num, input_string.as_bytes())
        }
    };
}

macro_rules! proxy {
    (post, $rpc: literal, $name: ident, $num: expr) => {
        proxy_post!($rpc, $name, $num)
    };
    (get, $rpc: literal, $name: ident, $num: expr) => {
        proxy_get!($rpc, $name, $num)
    };
}

enum ReadData {
    Ok(Vec<u8>),
    IoError,
    PayloadTooLarge,
}

async fn read_data(data: Data<'_>, limit: ByteUnit) -> ReadData {
    let stream = data.open(limit);
    let data = match stream.into_bytes().await {
        Ok(data) => data,
        Err(_) => return ReadData::IoError,
    };
    if !data.is_complete() {
        return ReadData::PayloadTooLarge;
    }
    ReadData::Ok(data.into_inner())
}

macro_rules! proxy_bin {
    ($rpc: literal, $name: ident, $num: expr) => {
        #[post($rpc, data = "<data>")]
        async fn $name(data: Data<'_>, limits: &Limits) -> JsonValue {
            let limit = limits.get(stringify!($name)).unwrap_or(100.mebibytes());
            let data = match read_data(data, limit).await {
                ReadData::Ok(data) => data,
                ReadData::IoError => {
                    return json!({
                        "status": "error",
                        "payload": "Io error: Read input data failed"
                    })
                }
                ReadData::PayloadTooLarge => {
                    return json!({
                        "status": "error",
                        "payload": "Entity too large"
                    })
                }
            };
            do_ecall_handle!($num, &data)
        }
    };
}

macro_rules! proxy_routes {
    ($(($m: ident, $rpc: literal, $name: ident, $num: expr),)+) => {{
        $(proxy!($m, $rpc, $name, $num);)+
        routes![$($name),+]
    }};
}

macro_rules! proxy_bin_routes {
    ($(($rpc: literal, $name: ident, $num: expr),)+) => {{
        $(proxy_bin!($rpc, $name, $num);)+
        routes![$($name),+]
    }};
}

#[post("/kick")]
fn kick() -> String {
    std::process::exit(0);
}

#[get("/info")]
fn getinfo() -> String {
    runtime::ecall_getinfo()
}

enum RpcType {
    Public,
    Private,
}

impl RpcType {
    fn is_public(&self) -> bool {
        match self {
            RpcType::Public => true,
            RpcType::Private => false,
        }
    }
}

fn rpc_type(method: &str) -> RpcType {
    use PhactoryAPIMethod::*;
    use RpcType::*;
    match PhactoryAPIMethod::from_str(method) {
        None => Private,
        Some(method) => match method {
            // Private RPCs
            SyncHeader => Private,
            SyncParaHeader => Private,
            SyncCombinedHeaders => Private,
            DispatchBlocks => Private,
            InitRuntime => Private,
            GetRuntimeInfo => Private,
            GetEgressMessages => Private,
            GetWorkerState => Private,
            AddEndpoint => Private,
            RefreshEndpointSigningTime => Private,
            GetEndpointInfo => Private,
            SignEndpointInfo => Private,
            DerivePhalaI2pKey => Private,
            Echo => Private,
            HandoverCreateChallenge => Private,
            HandoverStart => Private,
            HandoverAcceptChallenge => Private,
            HandoverReceive => Private,
            ConfigNetwork => Private,
            HttpFetch => Private,
            GetNetworkConfig => Private,
            LoadChainState => Private,
            Stop => Private,
            LoadStorageProof => Private,
            TakeCheckpoint => Private,
            // Public RPCs
            GetInfo => Public,
            ContractQuery => Public,
            GetContractInfo => Public,
            GetClusterInfo => Public,
            UploadSidevmCode => Public,
            CalculateContractId => Public,
        },
    }
}

fn default_payload_limit_for_method(method: PhactoryAPIMethod) -> ByteUnit {
    use PhactoryAPIMethod::*;

    match method {
        GetInfo => 1.kibibytes(),
        SyncHeader => 100.mebibytes(),
        SyncParaHeader => 100.mebibytes(),
        SyncCombinedHeaders => 100.mebibytes(),
        DispatchBlocks => 100.mebibytes(),
        InitRuntime => 10.mebibytes(),
        GetRuntimeInfo => 1.kibibytes(),
        GetEgressMessages => 1.kibibytes(),
        ContractQuery => 500.kibibytes(),
        GetWorkerState => 1.kibibytes(),
        AddEndpoint => 10.kibibytes(),
        RefreshEndpointSigningTime => 10.kibibytes(),
        GetEndpointInfo => 1.kibibytes(),
        DerivePhalaI2pKey => 10.kibibytes(),
        Echo => 1.mebibytes(),
        HandoverCreateChallenge => 10.kibibytes(),
        HandoverStart => 10.kibibytes(),
        HandoverAcceptChallenge => 10.kibibytes(),
        HandoverReceive => 10.kibibytes(),
        SignEndpointInfo => 32.kibibytes(),
        ConfigNetwork => 10.kibibytes(),
        HttpFetch => 100.mebibytes(),
        GetContractInfo => 100.kibibytes(),
        GetClusterInfo => 1.kibibytes(),
        UploadSidevmCode => 32.mebibytes(),
        CalculateContractId => 1.kibibytes(),
        GetNetworkConfig => 1.kibibytes(),
        LoadChainState => 500.mebibytes(),
        Stop => 1.kibibytes(),
        LoadStorageProof => 10.mebibytes(),
        TakeCheckpoint => 1.kibibytes(),
    }
}

fn limit_for_method(method: &str, limits: &Limits) -> ByteUnit {
    if let Some(v) = limits.get(method) {
        return v;
    }
    match PhactoryAPIMethod::from_str(method) {
        None => 1.mebibytes(),
        Some(method) => default_payload_limit_for_method(method),
    }
}

#[post("/<method>?<json>", data = "<data>")]
async fn prpc_proxy(
    method: String,
    data: Data<'_>,
    limits: &Limits,
    content_type: Option<&ContentType>,
    json: bool,
) -> Custom<Vec<u8>> {
    let limit = limit_for_method(&method, limits);
    let data = match read_data(data, limit).await {
        ReadData::Ok(data) => data,
        ReadData::IoError => {
            return Custom(Status::ServiceUnavailable, b"Read body failed".to_vec());
        }
        ReadData::PayloadTooLarge => {
            return Custom(Status::PayloadTooLarge, b"Entity too large".to_vec());
        }
    };
    let json = json || content_type.map(|t| t.is_json()).unwrap_or(false);
    prpc_call(method, &data, json).await
}

async fn prpc_call(method: String, data: &[u8], json: bool) -> Custom<Vec<u8>> {
    let (status_code, output) = runtime::ecall_prpc_request(method, data, json).await;
    if let Some(status) = Status::from_code(status_code) {
        Custom(status, output)
    } else {
        error!("prpc: Invalid status code: {}!", status_code);
        Custom(Status::ServiceUnavailable, vec![])
    }
}

#[post("/<method>?<json>", data = "<data>")]
async fn prpc_proxy_acl(
    method: String,
    data: Data<'_>,
    limits: &Limits,
    content_type: Option<&ContentType>,
    json: bool,
) -> Custom<Vec<u8>> {
    info!("prpc_acl: request {}:", method);
    if !rpc_type(&method).is_public() {
        error!("prpc_acl: access denied");
        return Custom(Status::Forbidden, vec![]);
    }
    prpc_proxy(method, data, limits, content_type, json).await
}

#[get("/<method>")]
async fn prpc_proxy_get_acl(method: String) -> Custom<Vec<u8>> {
    info!("prpc_acl: get {}:", method);
    if !rpc_type(&method).is_public() {
        error!("prpc_acl: access denied");
        return Custom(Status::Forbidden, vec![]);
    }
    prpc_call(method, b"", true).await
}

#[get("/<method>")]
async fn prpc_proxy_get(method: String) -> Custom<Vec<u8>> {
    prpc_call(method, b"", true).await
}

fn cors_options() -> CorsOptions {
    let allowed_origins = AllowedOrigins::all();
    let allowed_methods: AllowedMethods = vec![Method::Get, Method::Post]
        .into_iter()
        .map(From::from)
        .collect();

    // You can also deserialize this
    rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods,
        allowed_headers: AllowedHeaders::all(),
        allow_credentials: true,
        ..Default::default()
    }
}

fn print_rpc_methods(prefix: &str, methods: &[&str]) {
    info!("Methods under {}:", prefix);
    for method in methods {
        info!("    {}", format!("{prefix}/{method}").blue());
    }
}

pub(super) fn rocket(args: &super::Args) -> rocket::Rocket<impl Phase> {
    let mut server = rocket::build()
        .mount(
            "/",
            proxy_routes![
                (get, "/get_info", get_info, actions::ACTION_GET_INFO),
                (post, "/get_info", get_info_post, actions::ACTION_GET_INFO),
            ],
        )
        .mount(
            "/bin_api",
            proxy_bin_routes![
                ("/sync_header", sync_header, actions::BIN_ACTION_SYNC_HEADER),
                (
                    "/dispatch_block",
                    dispatch_block,
                    actions::BIN_ACTION_DISPATCH_BLOCK
                ),
                (
                    "/sync_para_header",
                    sync_para_header,
                    actions::BIN_ACTION_SYNC_PARA_HEADER
                ),
                (
                    "/sync_combined_headers",
                    sync_combined_headers,
                    actions::BIN_ACTION_SYNC_COMBINED_HEADERS
                ),
            ],
        )
        .mount("/", routes![getinfo]);

    if args.enable_kick_api {
        info!("ENABLE `kick` API");

        server = server.mount("/", routes![kick]);
    }

    server = server.mount("/prpc", routes![prpc_proxy, prpc_proxy_get]);
    print_rpc_methods("/prpc", prpc::phactory_api_server::supported_methods());

    if args.allow_cors {
        info!("Allow CORS");

        server = server
            .mount("/", rocket_cors::catch_all_options_routes()) // mount the catch all routes
            .attach(cors_options().to_cors().expect("To not fail"))
            .manage(cors_options().to_cors().expect("To not fail"));
    }

    if args.measure_rpc_time {
        info!("Attaching time meter");
        server = server.attach(phala_rocket_middleware::TimeMeter);
    }

    server
}

/// api endpoint with access control, will be exposed to the public
pub(super) fn rocket_acl(args: &super::Args) -> Option<rocket::Rocket<impl Phase>> {
    let public_port: u16 = if args.public_port.is_some() {
        args.public_port.expect("public_port should be set")
    } else {
        return None;
    };

    let figment = rocket::Config::figment()
        .merge(("address", "0.0.0.0"))
        .merge(("port", public_port))
        .merge(("limits", Limits::new().limit("json", 100.mebibytes())));

    let mut server_acl = rocket::custom(figment).mount("/", routes![getinfo]);

    server_acl = server_acl.mount("/prpc", routes![prpc_proxy_acl, prpc_proxy_get_acl]);

    if args.allow_cors {
        info!("Allow CORS");

        server_acl = server_acl
            .mount("/", rocket_cors::catch_all_options_routes()) // mount the catch all routes
            .attach(cors_options().to_cors().expect("To not fail"))
            .manage(cors_options().to_cors().expect("To not fail"));
    }

    let signer = ResponseSigner::new(1024 * 1024 * 10, runtime::ecall_sign_http_response);
    server_acl = server_acl.attach(signer);

    Some(server_acl)
}
