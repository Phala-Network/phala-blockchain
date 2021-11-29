use anyhow::{anyhow, Context, Error, Result};
#[allow(unused_imports)]
use log::{debug, error, info, warn};

use rocket::http::hyper::Error::Status;
use rocket::http::Method;
use rocket_contrib::json::{Json, JsonValue};
use rocket_cors::{AllowedHeaders, AllowedMethods, AllowedOrigins, CorsOptions};
use serde_json::ser::State;
use std::env;
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
    target_ident: &String,
    path: &String,
    method: &types::PRouterRequestMethod,
    data: &Vec<u8>,
) -> Result<Vec<u8>> {
    // http is enough since i2p will encrypt msg anyway.
    let target_url = format!("http://{}.b32.i2p:8000{}", target_ident, path);
    let client = reqwest::blocking::Client::builder()
        .proxy(reqwest::Proxy::http("http://localhost:4444/")?)
        .build()?;

    debug!(
        "PRouter send data to {}, method: {:?}",
        &target_url, &method
    );
    let mut res = match method {
        types::PRouterRequestMethod::GET => client.get(target_url).send()?,
        types::PRouterRequestMethod::POST => client.post(target_ident).body(data.clone()).send()?,
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
    debug!("{:?}", prouter_send_json);
    let mut response = types::PRouterSendJsonResponse {
        status: 0,
        msg: Default::default(),
    };

    let Json(prouter_send_data) = prouter_send_json;
    let para_api = para_api_arc.lock().unwrap();

    match translator::block_get_pnetwork_ident_by_pk(
        &mut para_api.as_ref().expect("guaranteed to be initialized"),
        prouter_send_data.target_pubkey,
    )
    .ok_or(anyhow!("Failed to fetch onchain storage"))
    {
        Ok(ident) => {
            // http is enough since i2p will encrypt msg anyway.
            match send_data(
                &ident,
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
    let mut server = rocket::ignite()
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
