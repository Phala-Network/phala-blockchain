use anyhow::{anyhow, Result};
use log::{debug, error, info};

use rocket::config::Limits;
use rocket::http::Method;
use rocket_contrib::json::Json;
use rocket_cors::{AllowedHeaders, AllowedMethods, AllowedOrigins, CorsOptions};
use std::io::Read;

use phala_types::EndpointType;

use crate::translator;
use crate::types;

use rocket::post;

struct PRouterState {
    local_proxy: String,
    para_api: super::SharedParachainApi,
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

fn send_data(
    target_endpoint_type: EndpointType,
    target_endpoint: &Vec<u8>,
    path: &String,
    method: &types::PRouterRequestMethod,
    data: &Vec<u8>,
    local_proxy: &String,
) -> Result<Vec<u8>> {
    let mut endpoint_str = String::from_utf8_lossy(target_endpoint).into_owned();

    let mut client_builder = reqwest::blocking::Client::builder();
    if matches!(EndpointType::I2P, target_endpoint_type) {
        client_builder = client_builder.proxy(reqwest::Proxy::http(local_proxy)?);
    }
    let client = client_builder.build()?;
    let target_endpoint_url = format!("{}{}", endpoint_str, path);

    debug!(
        "PRouter send data to {}, method: {:?}",
        &target_endpoint_url, &method
    );

    let mut res = match method {
        types::PRouterRequestMethod::GET => client.get(target_endpoint_url).send()?,
        types::PRouterRequestMethod::POST => {
            client.post(target_endpoint_url).body(data.clone()).send()?
        }
    };

    if res.status().is_success() {
        let mut msg = Vec::new();
        res.read_to_end(&mut msg)?;
        Ok(msg)
    } else {
        Err(anyhow::Error::msg(res.status()))
    }
}

// by posting to this endpoint, data will be routed to the target endpoint
#[post("/json/send_data", format = "json", data = "<prouter_send_json>")]
fn json_send_data(
    prouter_state: rocket::State<PRouterState>,
    prouter_send_json: Json<types::PRouterSendJsonRequest>,
) -> Json<types::PRouterSendJsonResponse> {
    debug!("{:?}", &prouter_send_json);
    let Json(prouter_send_data) = prouter_send_json;
    let mut response = types::PRouterSendJsonResponse {
        status: 0,
        msg: Default::default(),
    };

    let para_api = prouter_state.para_api.lock().unwrap();
    // TODO(soptq): Query storage from the chain RPC might be slow sometime, which might impact the performance of our i2p data transfer speed.
    match translator::block_get_endpoint_info_by_pubkey(
        &mut para_api.as_ref().expect("guaranteed to be initialized"),
        prouter_send_data.target_pubkey,
    )
    .ok_or(anyhow!("Failed to fetch on-chain storage"))
    {
        Ok((endpoint_type, endpoint)) => {
            match send_data(
                endpoint_type,
                &endpoint,
                &prouter_send_data.path,
                &prouter_send_data.method,
                &prouter_send_data.data,
                &prouter_state.local_proxy,
            ) {
                Ok(msg) => {
                    response.msg = msg;
                }
                Err(e) => {
                    error!("send_data error: {}", e);
                    response.status = 1;
                }
            }
        }
        Err(e) => {
            error!("json_send_data error: {}", e);
            response.status = 1;
        }
    }

    Json(response)
}

pub fn rocket(
    local_proxy: String,
    para_api: super::SharedParachainApi,
    server_address: String,
    server_port: u16,
) -> rocket::Rocket {
    let cfg = rocket::config::Config::build(rocket::config::Environment::active().unwrap())
        .address(server_address)
        .port(server_port)
        .limits(Limits::new().limit("json", 104857600))
        .expect("Config should be build with no erros");

    let prouter_state = PRouterState {
        local_proxy,
        para_api,
    };

    let server = rocket::custom(cfg)
        .manage(prouter_state)
        .mount("/", rocket::routes!(json_send_data));
    info!("Allow CORS");
    server
        .mount("/", rocket_cors::catch_all_options_routes()) // mount the catch all routes
        .attach(cors_options().to_cors().expect("To not fail"))
        .manage(cors_options().to_cors().expect("To not fail"))
}
