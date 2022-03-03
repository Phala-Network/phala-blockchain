use clap::{Parser, Subcommand};

use podtracker::prpc::podtracker_api_client::PodtrackerApiClient;
use prpc::client::RequestClient;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version)]
#[clap(about = "A tool talking the podtracker")]
struct Args {
    #[clap(short, long, default_value_t = 8100)]
    port: u16,

    #[clap(subcommand)]
    subcommand: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Show the podtracker status
    Status,
    /// List all pods status
    List,
    /// Create a new pod
    New {
        /// The docker image used to create the pod
        #[clap(long)]
        image: String,
        /// Give the UUID of the pod
        #[clap(long)]
        id: String,
    },
    /// Stop and delete an existing pod
    Stop { id: String },
}

struct HttpClient {
    base_url: String,
}

fn from_debug(e: impl std::fmt::Debug) -> prpc::client::Error {
    prpc::client::Error::RpcError(format!("{:?}", e))
}

impl RequestClient for HttpClient {
    fn request(&self, path: &str, body: Vec<u8>) -> Result<Vec<u8>, prpc::client::Error> {
        let url = format!("{}/{}", self.base_url, path);
        let client = reqwest::blocking::Client::new();
        let response = client.post(&url).body(body).send().map_err(from_debug)?;
        response
            .bytes()
            .map(|res| res.as_ref().to_vec())
            .map_err(from_debug)
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let client = PodtrackerApiClient::new(HttpClient {
        base_url: format!("http://127.0.0.1:{}/prpc", args.port),
    });
    match args.subcommand {
        Command::Status => {
            let info = client.status(())?;
            println!("{:#?}", info);
        }
        Command::List => {
            println!("List");
        }
        Command::New { image, id } => {
            println!("Create {} {}", image, id);
        }
        Command::Stop { id } => {
            println!("Stop {}", id);
        }
    }
    Ok(())
}
