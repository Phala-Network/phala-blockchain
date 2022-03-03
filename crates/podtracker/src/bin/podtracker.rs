use clap::Parser;
use rocket::{custom, data::ToByteUnit, http::Status, post, response::status, routes, Data, State};

use tokio::sync::Mutex;

use log::error;
use podtracker::{
    prpc::{
        podtracker_api_server::{PodtrackerApi, PodtrackerApiServer},
        TrackerInfo,
    },
    Tracker,
};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The docker control socket used to talk to docker
    #[clap(long, default_value = "unix:///var/run/docker.sock")]
    docker_host: String,

    /// The port to serve the podtracker RPC server
    #[clap(long, default_value_t = 8100)]
    rpc_port: u16,

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
        return Err(anyhow::anyhow!(
            "Range too large. At most {} ports",
            max_ports
        ));
    }
    Ok((start, end))
}

struct App {
    tracker: Mutex<Tracker>,
}

#[post("/<method>", data = "<data>")]
async fn prpc_proxy(
    state: &State<App>,
    method: String,
    data: Data<'_>,
) -> Result<Vec<u8>, status::Custom<String>> {
    let body = data
        .open(2_u32.mebibytes())
        .into_bytes()
        .await
        .or(Err(status::Custom(
            Status::BadRequest,
            "Failed to read request body".into(),
        )))?;

    if !body.is_complete() {
        return Err(status::Custom(
            Status::BadRequest,
            "Request body too large".into(),
        ));
    }

    let body = body.into_inner();

    let mut server = PodtrackerApiServer::new(RpcHandler {
        tracker: &state.tracker,
    });

    match server.dispatch_request(&method, &body).await {
        Ok(response) => Ok(response),
        Err(err) => {
            error!("{}", err);
            Err(status::Custom(
                Status::InternalServerError,
                format!("{}", err),
            ))
        }
    }
}

struct RpcHandler<'a> {
    tracker: &'a Mutex<Tracker>,
}

#[::async_trait::async_trait]
impl PodtrackerApi for RpcHandler<'_> {
    async fn get_info(&mut self, _request: ()) -> Result<TrackerInfo, prpc::server::Error> {
        let info = self.tracker.lock().await.info();
        Ok(TrackerInfo {
            pods_running: info.pods_running as _,
            pods_allocated: info.pods_allocated as _,
            tcp_ports_available: info.tcp_ports_available as _,
        })
    }
}

#[rocket::main]
async fn main() {
    let args = Args::parse();
    let docker =
        docker_api::Docker::new(args.docker_host).expect("Unable to connect to docker service");
    let tracker = Tracker::new(docker, args.tcp_port_range);
    let app = App {
        tracker: Mutex::new(tracker),
    };
    let config = rocket::Config::figment().merge(("port", args.rpc_port));

    custom(config)
        .manage(app)
        .mount("/prpc", routes![prpc_proxy])
        .launch()
        .await
        .expect("Unable to launch rocket");
}
