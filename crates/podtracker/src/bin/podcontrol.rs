
use clap::{Parser, Subcommand};

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
    Stop {
        id: String,
    },
}



#[tokio::main]
async fn main() {
    let args = Args::parse();
    match args.subcommand {
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
}
