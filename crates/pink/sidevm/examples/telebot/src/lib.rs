use futures::StreamExt;
use telegram_bot::*;
use log::info;

#[sidevm::main]
async fn main() {
    use sidevm::logger::Logger;
    Logger::with_max_level(log::LevelFilter::Trace).init();

    info!("Starting up...");

    let token = "<Put your token here>";
    let api = Api::new(token);

    // Fetch new updates via long poll method
    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        // If the received update contains a new message...
        let update = update.expect("Failed to get update");
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { ref data, .. } = message.kind {
                // Answer message with "Hi".
                api.send(message.text_reply(format!(
                    "Hi, {}! You just wrote '{}'",
                    &message.from.first_name, data
                )))
                .await
                .expect("Failed to send message");
            }
        }
    }
}
