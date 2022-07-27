use rocket::data::ToByteUnit;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::{post, routes};
use rocket::{Data, State};
use scale::Decode;
use sp_core::crypto::AccountId32;
use std::str::FromStr;

use pink_sidevm_host_runtime::service::{Command, CommandSender, SystemMessage};
struct App {
    command_sender: CommandSender,
}

async fn read_data(data: Data<'_>) -> Option<Vec<u8>> {
    let stream = data.open(100.mebibytes());
    let data = stream.into_bytes().await.ok()?;
    Some(data.into_inner())
}

#[post("/push/message", data = "<data>")]
async fn push_message(app: &State<App>, data: Data<'_>) -> Result<(), Custom<&'static str>> {
    let body = read_data(data)
        .await
        .ok_or(Custom(Status::BadRequest, "No message payload"))?;
    app.command_sender
        .send(Command::PushMessage(body))
        .await
        .or(Err(Custom(
            Status::InternalServerError,
            "Failed to send message to the VM",
        )))?;
    Ok(())
}

#[post("/push/sysmessage", data = "<data>")]
async fn push_sys_message(app: &State<App>, data: Data<'_>) -> Result<(), Custom<&'static str>> {
    let body = read_data(data)
        .await
        .ok_or(Custom(Status::BadRequest, "No message payload"))?;
    let message = SystemMessage::decode(&mut &body[..]).or(Err(Custom(
        Status::BadRequest,
        "Failed to decode the message",
    )))?;
    app.command_sender
        .send(Command::PushSystemMessage(message))
        .await
        .or(Err(Custom(
            Status::InternalServerError,
            "Failed to send message to the VM",
        )))?;
    Ok(())
}

#[post("/push/query", data = "<data>")]
async fn push_query_no_origin(
    app: &State<App>,
    data: Data<'_>,
) -> Result<Vec<u8>, Custom<&'static str>> {
    push_query(app, None, data).await
}

#[post("/push/query/<origin>", data = "<data>")]
async fn push_query(
    app: &State<App>,
    origin: Option<&str>,
    data: Data<'_>,
) -> Result<Vec<u8>, Custom<&'static str>> {
    push_query_impl(app, origin, data).await
}

async fn push_query_impl(
    app: &State<App>,
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
                .into()
        ),
    };

    app.command_sender
        .send(Command::PushQuery {
            origin,
            payload,
            reply_tx,
        })
        .await
        .or(Err(Custom(
            Status::InternalServerError,
            "Failed to send query to the VM",
        )))?;
    let reply = rx.await.or(Err(Custom(
        Status::InternalServerError,
        "Failed to receive query reply from the VM",
    )))?;
    Ok(reply)
}

pub async fn serve(command_sender: CommandSender) -> anyhow::Result<()> {
    let _rocket = rocket::build()
        .manage(App { command_sender })
        .mount(
            "/",
            routes![
                push_message,
                push_sys_message,
                push_query,
                push_query_no_origin
            ],
        )
        .launch()
        .await?;
    Ok(())
}
