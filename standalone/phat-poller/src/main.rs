use clap::Parser;

mod app;
mod args;
mod contracts;
mod instant;
mod query;
mod web_api;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = args::AppArgs::parse();
    match args.action {
        args::Action::Run(args) => {
            let app = app::create_app(args);
            tokio::select! {
                r = web_api::serve(app.clone()) => r?,
                r = app.run() => r?,
            }
        }
    }
    Ok(())
}
