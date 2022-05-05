use std::str;

use rocket::data::Data;
use rocket::data::ToByteUnit;
use rocket::http::Method;
use rocket::http::Status;
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

async fn read_data(data: Data<'_>) -> Option<Vec<u8>> {
    let stream = data.open(100.mebibytes());
    let data = stream.into_bytes().await.ok()?;
    Some(data.into_inner())
}

macro_rules! proxy_bin {
    ($rpc: literal, $name: ident, $num: expr) => {
        #[post($rpc, data = "<data>")]
        async fn $name(data: Data<'_>) -> JsonValue {
            let data = match read_data(data).await {
                Some(data) => data,
                None => {
                    return json!({
                        "status": "error",
                        "payload": "Io error: Read input data failed"
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
fn kick() {
    std::process::exit(0);
}

#[post("/<method>", data = "<data>")]
async fn prpc_proxy(method: String, data: Data<'_>) -> Custom<Vec<u8>> {
    let path_bytes = method.as_bytes();
    let data = match read_data(data).await {
        Some(data) => data,
        None => {
            return Custom(Status::BadRequest, b"Read body failed".to_vec());
        }
    };

    let (status_code, output) = runtime::ecall_prpc_request(path_bytes, &data);
    if let Some(status) = Status::from_code(status_code) {
        Custom(status, output)
    } else {
        error!("prpc: Invalid status code: {}!", status_code);
        Custom(Status::ServiceUnavailable, vec![])
    }
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
        info!("    {}", format!("{}/{}", prefix, method).blue());
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
        );

    if args.enable_kick_api {
        info!("ENABLE `kick` API");

        server = server.mount("/", routes![kick]);
    }

    server = server.mount("/prpc", routes![prpc_proxy]);
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
