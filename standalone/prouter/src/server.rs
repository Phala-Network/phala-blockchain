use anyhow::{anyhow, Context, Error, Result};
#[allow(unused_imports)]
use log::{debug, error, info, warn};

use rocket::config::Limits;
use rocket::http::hyper::Error::Status;
use rocket::http::Method;
use rocket_contrib::json::{Json, JsonValue};
use rocket_cors::{AllowedHeaders, AllowedMethods, AllowedOrigins, CorsOptions};
use serde_json::ser::State;
use std::env;
use std::fmt::format;
use std::io::Read;
use std::sync::{Arc, Mutex};

use crate::translator;
use crate::types;

use crate::subxt::sp_runtime::biguint::mul_single;
use crate::types::PRouterRequestMethod;
use phaxt::{ParachainApi, RelaychainApi};

lazy_static! {
    static ref ALLOW_CORS: bool = env::var("ALLOW_CORS").unwrap_or_else(|_| "".to_string()) != "";
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
    target_endpoint: &Vec<u8>,
    path: &String,
    method: &types::PRouterRequestMethod,
    data: &Vec<u8>,
) -> Result<Vec<u8>> {
    let mut endpoint_str = String::from_utf8_lossy(target_endpoint).into_owned();

    let mut client_builder = reqwest::blocking::Client::builder();
    if endpoint_str.contains("b32.i2p") {
        // a i2pd address, http is enough since i2p will encrypt msg anyway
        endpoint_str = format!("http://{}", endpoint_str);
        client_builder = client_builder.proxy(reqwest::Proxy::http("http://localhost:4444/")?)
    }
    let client = client_builder.build()?;

    let target_url = format!("{}{}", endpoint_str, path);
    debug!(
        "PRouter send data to {}, method: {:?}",
        &target_url, &method
    );
    let mut res = match method {
        types::PRouterRequestMethod::GET => client.get(target_url).send()?,
        types::PRouterRequestMethod::POST => client.post(target_url).body(data.clone()).send()?,
    };

    if res.status().is_success() {
        let mut msg = Vec::new();
        res.read_to_end(&mut msg)?;
        Ok(msg)
    } else {
        Err(anyhow::Error::msg(res.status()))
    }
}

#[post("/json/send_data", format = "json", data = "<prouter_send_json>")]
fn json_send_data(
    para_api_arc: rocket::State<Arc<Mutex<Option<ParachainApi>>>>,
    prouter_send_json: Json<types::PRouterSendJsonRequest>,
) -> Json<types::PRouterSendJsonResponse> {
    debug!("{:?}", &prouter_send_json);
    let Json(prouter_send_data) = prouter_send_json;
    let mut response = types::PRouterSendJsonResponse {
        status: 0,
        msg: Default::default(),
    };

    let para_api = para_api_arc.lock().unwrap();
    match translator::block_get_endpoint_by_pubkey(
        &mut para_api.as_ref().expect("guaranteed to be initialized"),
        prouter_send_data.target_pubkey,
    )
    .ok_or(anyhow!("Failed to fetch on-chain storage"))
    {
        Ok(endpoint) => {
            match send_data(
                &endpoint,
                &prouter_send_data.path,
                &prouter_send_data.method,
                &prouter_send_data.data,
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
    api: Arc<Mutex<Option<RelaychainApi>>>,
    para_api: Arc<Mutex<Option<ParachainApi>>>,
) -> rocket::Rocket {
    let cfg = rocket::config::Config::build(rocket::config::Environment::Development)
        .address("127.0.0.1")
        .port(8001)
        .workers(1)
        .limits(Limits::new().limit("json", 104857600))
        .expect("Config should be build with no erros");

    let mut server = rocket::custom(cfg)
        .manage(api)
        .manage(para_api)
        .mount("/", rocket::routes!(json_send_data));

    if *ALLOW_CORS {
        info!("Allow CORS");

        server
            .mount("/", rocket_cors::catch_all_options_routes()) // mount the catch all routes
            .attach(cors_options().to_cors().expect("To not fail"))
            .manage(cors_options().to_cors().expect("To not fail"))
    } else {
        server
    }
}
