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
        args::Action::Eval(args::EvalArgs {
            worker_uri,
            caller,
            contract,
            js_file,
        }) => {
            use phactory_api::pruntime_client::new_pruntime_client_no_log;
            let worker = new_pruntime_client_no_log(worker_uri.clone());
            let info = worker.get_info(()).await?;
            println!("Worker info: {:?}", info.memory_usage);
            let pubkey = info
                .public_key
                .ok_or_else(|| anyhow::anyhow!("Worker has no public key"))?;
            let pubkey = hex::decode(pubkey.as_str().trim_start_matches("0x"))?;

            if js_file.ends_with(".js") {
                let js_code = std::fs::read_to_string(js_file)?;
                let response = query::pink_query::<_, String>(
                    &pubkey,
                    &worker_uri,
                    contract,
                    contracts::SELECTOR_EVAL,
                    js_code,
                    &caller,
                )
                .await?;
                println!("{}", response);
            } else {
                match js_file.as_str() {
                    "ver" => {
                        let response = query::pink_query::<_, (u16, u16, u16)>(
                            &pubkey,
                            &worker_uri,
                            contract,
                            0x317f6bf3,
                            (),
                            &caller,
                        )
                        .await?;
                        println!("{:?}", response);
                    }
                    _ => {
                        println!("Invalid js file: {}", js_file);
                    }
                }
            }
        }
    }
    Ok(())
}
