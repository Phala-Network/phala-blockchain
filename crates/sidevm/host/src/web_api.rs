use rocket::data::ToByteUnit;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::{get, post, routes};
use rocket::{Data, State};
use tracing::{info, warn};

use scale::Decode;
use sidevm_host_runtime::ShortId;
use sp_core::crypto::AccountId32;
use tokio::task::JoinHandle;

use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

use sidevm::{Command, CommandSender, Spawner, SystemMessage};
use sidevm_host_runtime::rocket_stream::{connect, RequestInfo, StreamResponse};
use sidevm_host_runtime::{
    service::{self as sidevm, ExitReason},
    OutgoingRequest,
};

use crate::Args;
struct VmHandle {
    sender: CommandSender,
    handle: JoinHandle<ExitReason>,
}
struct AppInner {
    next_id: u32,
    instances: HashMap<u32, VmHandle>,
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
        self.sender_for(vmid)
            .await
            .ok_or((404, "Instance not found"))?
            .send(message)
            .await
            .or(Err((500, "Failed to send message")))?;
        Ok(())
    }

    async fn sender_for(&self, vmid: u32) -> Option<Sender<Command>> {
        Some(self.inner.lock().await.instances.get(&vmid)?.sender.clone())
    }

    async fn take_handle(&self, vmid: u32) -> Option<VmHandle> {
        self.inner.lock().await.instances.remove(&vmid)
    }

    async fn run_wasm(
        &self,
        wasm_bytes: Vec<u8>,
        weight: u32,
        id: Option<u32>,
    ) -> Result<u32, &'static str> {
        let mut inner = self.inner.lock().await;
        let id = match id {
            Some(id) => id,
            None => inner.next_id,
        };
        inner.next_id = id
            .checked_add(1)
            .ok_or("Too many instances")?
            .max(inner.next_id);

        let mut vmid = [0u8; 32];

        vmid[0..4].copy_from_slice(&id.to_be_bytes());

        println!("VM {id} running...");
        let (sender, handle) = inner
            .spawner
            .start(
                &wasm_bytes,
                inner.args.max_memory_pages,
                vmid,
                inner.args.gas_per_breath,
                crate::simple_cache(),
                weight,
                None,
            )
            .unwrap();
        inner.instances.insert(id, VmHandle { sender, handle });
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

#[post("/sidevm/<id>/<path..>", data = "<body>")]
async fn connect_vm_post<'r>(
    app: &State<App>,
    head: RequestInfo,
    id: u32,
    path: PathBuf,
    body: Data<'r>,
) -> Result<StreamResponse, (Status, String)> {
    connect_vm(app, head, id, path, Some(body)).await
}

#[get("/sidevm/<id>/<path..>")]
async fn connect_vm_get<'r>(
    app: &State<App>,
    head: RequestInfo,
    id: u32,
    path: PathBuf,
) -> Result<StreamResponse, (Status, String)> {
    connect_vm(app, head, id, path, None).await
}

async fn connect_vm<'r>(
    app: &State<App>,
    head: RequestInfo,
    id: u32,
    path: PathBuf,
    body: Option<Data<'r>>,
) -> Result<StreamResponse, (Status, String)> {
    let Some(command_tx) = app.sender_for(id).await else {
        return Err((Status::NotFound, Default::default()));
    };
    let path = path
        .to_str()
        .ok_or((Status::BadRequest, "Invalid path".to_string()))?;
    let result = connect(head, path, body, command_tx).await;
    match result {
        Ok(response) => Ok(response),
        Err(err) => Err((Status::InternalServerError, err.to_string())),
    }
}

#[post("/run?<weight>&<id>", data = "<data>")]
async fn run(
    app: &State<App>,
    weight: Option<u32>,
    id: Option<u32>,
    data: Data<'_>,
) -> Result<String, Custom<&'static str>> {
    if let Some(id) = id {
        if let Some(handle) = app.take_handle(id).await {
            info!("Stopping VM {id}...");
            if let Err(err) = handle.sender.send(Command::Stop).await {
                warn!("Failed to send stop command to the VM: {err:?}");
            }
            match handle.handle.await {
                Ok(reason) => info!("VM exited: {reason:?}"),
                Err(err) => warn!("Failed to wait VM exit: {err:?}"),
            }
        };
    }
    let code = read_data(data)
        .await
        .ok_or(Custom(Status::BadRequest, "No message payload"))?;
    let id = app
        .run_wasm(code, weight.unwrap_or(1), id)
        .await
        .map_err(|reason| Custom(Status::InternalServerError, reason))?;
    Ok(id.to_string())
}

#[post("/stop?<id>")]
async fn stop(app: &State<App>, id: u32) -> Result<(), Custom<&'static str>> {
    let Some(handle) = app.take_handle(id).await else {
        return Err(Custom(Status::NotFound, "Instance not found"));
    };
    info!("Stopping VM {id}...");
    if let Err(err) = handle.sender.send(Command::Stop).await {
        warn!("Failed to send stop command to the VM: {err:?}");
    }
    match handle.handle.await {
        Ok(reason) => info!("VM exited: {reason:?}"),
        Err(err) => warn!("Failed to wait VM exit: {err:?}"),
    }
    Ok(())
}

#[get("/info")]
async fn info(app: &State<App>) -> String {
    let inner = app.inner.lock().await;
    serde_json::json!({
        "running": sidevm_host_runtime::vm_count(),
        "deployed": inner.instances.len(),
        "ids": inner.instances.keys().cloned().collect::<Vec<_>>(),
    })
    .to_string()
}

pub async fn serve(args: Args) -> anyhow::Result<()> {
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);
    let (run, spawner) = sidevm::service(args.workers, tx);
    tokio::spawn(async move {
        while let Some((id, message)) = rx.recv().await {
            let OutgoingRequest::Query {
                contract_id,
                payload,
                reply_tx,
            } = message;
            let vmid = ShortId(id);
            let dest = ShortId(contract_id);
            info!(%vmid, "Outgoing message to {dest} payload: {payload:?}");
            _ = reply_tx.send(Vec::new());
        }
    });
    std::thread::spawn(move || {
        run.blocking_run(|evt| {
            println!("event: {:?}", evt);
        });
    });
    let program = args.program.clone();
    let app = App::new(spawner, args);
    if let Some(program) = program {
        let wasm_codes = std::fs::read(&program)?;
        app.run_wasm(wasm_codes, 1, None)
            .await
            .map_err(|reason| anyhow::anyhow!("Failed to run wasm: {}", reason))?;
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
                stop,
                connect_vm_get,
                connect_vm_post,
                info,
            ],
        )
        .launch()
        .await?;
    Ok(())
}
