use std::sync::Arc;

use clap::Parser;
use rocket::{build, get, post, routes, Data, State};
use tokio::sync::Mutex;

use podtracker::Tracker;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The docker control socket used to talk to docker
    #[clap(long, default_value = "unix:///var/run/docker.sock")]
    docker_host: String,

    /// The address to listen on
    #[clap(long, default_value = "127.0.0.1:8100")]
    listen: String,

    /// The TCP port range to be allocated to pods
    #[clap(long, default_value = "8800:8899", parse(try_from_str=parse_port_range))]
    tcp_port_range: (u16, u16),
}

fn parse_port_range(s: &str) -> anyhow::Result<(u16, u16)> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Must be in the form of 'start:end'"));
    }
    let start = parts[0].parse::<u16>()?;
    let end = parts[1].parse::<u16>()?;
    if start > end {
        return Err(anyhow::anyhow!("Start port must be less than end port"));
    }
    let max_ports = 1000;
    if end - start >= max_ports {
        return Err(anyhow::anyhow!("Range too large. At most {} ports", max_ports));
    }
    Ok((start, end))
}

struct App {
    tracker: Arc<Mutex<Tracker>>,
}

#[get("/pods")]
async fn get_pods(state: &State<App>) {
    let tracker = state.tracker.lock().await;
}

#[post("/create_pod")]
async fn create_pod(state: &State<App>) {
    let tracker = state.tracker.lock().await;
}

#[rocket::main]
async fn main() {
    let args = Args::parse();
    let docker =
        docker_api::Docker::new(args.docker_host).expect("Unable to connect to docker service");
    let tracker = Tracker::new(docker, args.tcp_port_range);
    let app = App {
        tracker: Arc::new(Mutex::new(tracker)),
    };
    build()
        .manage(app)
        .mount("/", routes![get_pods])
        .launch()
        .await
        .expect("Unable to launch rocket");
}
