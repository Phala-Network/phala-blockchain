use std::str;
use std::io::BufWriter;

use rocket::config::Limits;
use rocket::data::Data;
use rocket::http::Method;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::response::Stream;
use rocket::{get, post, routes};
use rocket_contrib::json;
use rocket_contrib::json::{Json, JsonValue};
use rocket_cors::{AllowedHeaders, AllowedMethods, AllowedOrigins, CorsOptions};

use colored::Colorize as _;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use os_pipe::PipeReader;

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

fn read_data(data: Data) -> Option<Vec<u8>> {
    use std::io::Read;
    let mut stream = data.open();
    let mut data = Vec::new();
    stream.read_to_end(&mut data).ok()?;
    Some(data)
}

macro_rules! proxy_bin {
    ($rpc: literal, $name: ident, $num: expr) => {
        #[post($rpc, data = "<data>")]
        fn $name(data: Data) -> JsonValue {
            let data = match read_data(data) {
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
fn prpc_proxy(method: String, data: Data) -> Custom<Vec<u8>> {
    let path_bytes = method.as_bytes();
    let data = match read_data(data) {
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

#[post("/<method>", data = "<data>")]
fn prpc_proxy_acl(method: String, data: Data) -> Custom<Vec<u8>> {
    info!("prpc_acl: request {}:", method);
    let permitted_method: [&str; 1] = ["contract_query"];
    if !permitted_method.contains(&&method[..]) {
        error!("prpc_acl: access denied");
        return Custom(Status::ServiceUnavailable, vec![]);
    }

    prpc_proxy(method, data)
}

#[get("/dump")]
fn dump_state() -> anyhow::Result<Stream<PipeReader>> {
    let (r, writer) = os_pipe::pipe()?;
    std::thread::spawn(move || {
        let writer = BufWriter::new(writer);
        if let Err(err) = crate::runtime::ecall_dump_state(writer) {
            error!("Failed to dump state: {:?}", err);
        }
    });
    Ok(r.into())
}

#[post("/load", data = "<data>")]
fn load_state(data: Data) -> Custom<()> {
    match crate::runtime::ecall_load_state(data.open()) {
        Ok(_) => Custom(Status::Ok, ()),
        Err(err) => {
            error!("Failed to load state: {:?}", err);
            Custom(Status::BadRequest, ())
        }
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

pub fn rocket(allow_cors: bool, enable_kick_api: bool) -> rocket::Rocket {
    let mut server = rocket::ignite()
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

    if enable_kick_api {
        info!("ENABLE `kick` API");

        server = server.mount("/", routes![kick]);
    }

    server = server.mount("/state", routes![dump_state, load_state]);

    server = server.mount("/prpc", routes![prpc_proxy]);
    print_rpc_methods("/prpc", prpc::phactory_api_server::supported_methods());

    if allow_cors {
        info!("Allow CORS");

        server
            .mount("/", rocket_cors::catch_all_options_routes()) // mount the catch all routes
            .attach(cors_options().to_cors().expect("To not fail"))
            .manage(cors_options().to_cors().expect("To not fail"))
    } else {
        server
    }
}

// api endpoint with access control, will be exposed to the public
pub fn rocket_acl(allow_cors: bool, port: u16) -> rocket::Rocket {
    let cfg = rocket::config::Config::build(rocket::config::Environment::active().unwrap())
        .address("0.0.0.0")
        .port(port)
        .workers(1)
        .limits(Limits::new().limit("json", 104857600))
        .expect("Config should be build with no erros");

    let mut server_acl = rocket::custom(cfg)
        .mount(
            "/",
            proxy_routes![
                (get, "/get_info", get_info, actions::ACTION_GET_INFO),
                (post, "/get_info", get_info_post, actions::ACTION_GET_INFO),
            ],
        );

    server_acl = server_acl.mount("/prpc", routes![prpc_proxy_acl]);

    if allow_cors {
        info!("Allow CORS");

        server_acl
            .mount("/", rocket_cors::catch_all_options_routes()) // mount the catch all routes
            .attach(cors_options().to_cors().expect("To not fail"))
            .manage(cors_options().to_cors().expect("To not fail"))
    } else {
        server_acl
    }
}
