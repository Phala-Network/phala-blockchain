use rocket::data::ToByteUnit;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::{post, routes};
use rocket::{Data, State};
use scale::Decode;
use sp_core::crypto::AccountId32;
use std::collections::HashMap;
use std::str::FromStr;
use tokio::sync::Mutex;

use sidevm_host_runtime::service as sidevm;
use sidevm::{Command, CommandSender, Spawner, SystemMessage};

use crate::Args;
struct AppInner {
    next_id: u32,
    instances: HashMap<u32, CommandSender>,
    args: Args,
    spawner: Spawner,
}

struct App {
    inner: Mutex<AppInner>,
}

impl App {
    fn new(spawner: Spawner, args: Args) -> Self {
        Self {
            inner: Mutex::new(AppInner {
                instances: HashMap::new(),
                next_id: 0,
                spawner,
                args,
            }),
        }
    }

    async fn send(&self, vmid: u32, message: Command) -> Result<(), (u16, &'static str)> {
        self.inner
            .lock()
            .await
            .instances
            .get(&vmid)
            .ok_or((404, "Instance not found"))?
            .send(message)
            .await
            .or(Err((500, "Failed to send message")))?;
        Ok(())
    }

    async fn run_wasm(&self, weight: u32, wasm_bytes: Vec<u8>) -> Result<u32, &'static str> {
        let mut inner = self.inner.lock().await;
        let id = inner.next_id;
        inner.next_id += 1;

        let mut vmid = [0u8; 32];

        vmid[0..4].copy_from_slice(&id.to_be_bytes());

        println!("VM {id} running...");
        let (sender, handle) = inner
            .spawner
            .start(
                &wasm_bytes,
                1024,
                vmid,
                inner.args.gas_per_breath,
                crate::simple_cache(),
                weight,
            )
            .unwrap();
        inner.instances.insert(id, sender);
        tokio::spawn(handle);
        Ok(id)
    }
}

async fn read_data(data: Data<'_>) -> Option<Vec<u8>> {
    let stream = data.open(10000.mebibytes());
    let data = stream.into_bytes().await.ok()?;
    Some(data.into_inner())
}

#[post("/push/message/<id>", data = "<data>")]
async fn push_message(
    app: &State<App>,
    id: u32,
    data: Data<'_>,
) -> Result<(), Custom<&'static str>> {
    let body = read_data(data)
        .await
        .ok_or(Custom(Status::BadRequest, "No message payload"))?;
    app.send(id, Command::PushMessage(body))
        .await
        .map_err(|(code, reason)| Custom(Status { code }, reason))?;
    Ok(())
}

#[post("/push/sysmessage/<id>", data = "<data>")]
async fn push_sys_message(
    app: &State<App>,
    id: u32,
    data: Data<'_>,
) -> Result<(), Custom<&'static str>> {
    let body = read_data(data)
        .await
        .ok_or(Custom(Status::BadRequest, "No message payload"))?;
    let message = SystemMessage::decode(&mut &body[..]).or(Err(Custom(
        Status::BadRequest,
        "Failed to decode the message",
    )))?;
    app.send(id, Command::PushSystemMessage(message))
        .await
        .map_err(|(code, reason)| Custom(Status { code }, reason))?;
    Ok(())
}

#[post("/push/query/<id>", data = "<data>")]
async fn push_query_no_origin(
    app: &State<App>,
    id: u32,
    data: Data<'_>,
) -> Result<Vec<u8>, Custom<&'static str>> {
    push_query(app, id, None, data).await
}

#[post("/push/query/<id>/<origin>", data = "<data>")]
async fn push_query(
    app: &State<App>,
    id: u32,
    origin: Option<&str>,
    data: Data<'_>,
) -> Result<Vec<u8>, Custom<&'static str>> {
    let payload = read_data(data)
        .await
        .ok_or(Custom(Status::BadRequest, "No message payload"))?;

    let (reply_tx, rx) = tokio::sync::oneshot::channel();
    let origin = match origin {
        None => None,
        Some(origin) => Some(
            AccountId32::from_str(origin)
                .or(Err(Custom(
                    Status::BadRequest,
                    "Failed to decode the origin",
                )))?
                .into(),
        ),
    };

    app.send(
        id,
        Command::PushQuery {
            origin,
            payload,
            reply_tx,
        },
    )
    .await
    .map_err(|(code, reason)| Custom(Status { code }, reason))?;
    let reply = rx.await.or(Err(Custom(
        Status::InternalServerError,
        "Failed to receive query reply from the VM",
    )))?;
    Ok(reply)
}

#[post("/run/<weight>", data = "<data>")]
async fn run(app: &State<App>, weight: u32, data: Data<'_>) -> Result<String, Custom<&'static str>> {
    let code = read_data(data)
        .await
        .ok_or(Custom(Status::BadRequest, "No message payload"))?;
    let id = app
        .run_wasm(weight, code)
        .await
        .map_err(|reason| Custom(Status::InternalServerError, reason))?;
    Ok(id.to_string())
}

pub async fn serve(args: Args) -> anyhow::Result<()> {
    let (run, spawner) = sidevm::service(args.workers);
    std::thread::spawn(move || {
        run.blocking_run(|evt| {
            println!("event: {:?}", evt);
        });
    });
    let program = args.program.clone();
    let app = App::new(spawner, args);
    if let Some(program) = program {
        let wasm_codes = std::fs::read(&program)?;
        app.run_wasm(1, wasm_codes).await.map_err(|reason| {
            anyhow::anyhow!("Failed to run wasm: {}", reason)
        })?;
    }
    let _rocket = rocket::build()
        .manage(app)
        .mount(
            "/",
            routes![
                push_message,
                push_sys_message,
                push_query,
                push_query_no_origin,
                run,
            ],
        )
        .launch()
        .await?;
    Ok(())
}
