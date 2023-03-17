mod api_server;
mod ias;
mod pal_gramine;
mod runtime;

use std::{env, thread};

use clap::Parser;
use tracing::{error, info, info_span, Instrument};

use phactory_api::ecall_args::{git_revision, InitArgs};

mod handover;
use phala_sanitized_logger as logger;

#[derive(Parser, Debug, Clone)]
#[clap(about = "The Phala TEE worker app.", version, author)]
struct Args {
    /// Number of CPU cores to be used for mining.
    #[arg(short, long)]
    cores: Option<u32>,

    /// Run benchmark at startup.
    #[arg(long)]
    init_bench: bool,

    /// Allow CORS for HTTP
    #[arg(long)]
    allow_cors: bool,

    /// Turn on /kick API
    #[arg(long)]
    enable_kick_api: bool,

    /// Listening IP address of HTTP
    #[arg(long)]
    address: Option<String>,

    /// Listening port of HTTP
    #[arg(long)]
    port: Option<u16>,

    /// Listening port of HTTP (with access control)
    #[arg(long)]
    public_port: Option<u16>,

    /// Disable checkpoint
    #[arg(long)]
    disable_checkpoint: bool,

    /// Checkpoint interval in seconds, default to 5 minutes
    #[arg(long)]
    #[arg(default_value_t = 300)]
    checkpoint_interval: u64,

    /// Remove corrupted checkpoint so that pruntime can restart to continue to load others.
    #[arg(long)]
    remove_corrupted_checkpoint: bool,

    /// Max number of checkpoint files kept
    #[arg(long)]
    #[arg(default_value_t = 5)]
    max_checkpoint_files: u32,

    /// Handover key from another running pruntime instance
    #[arg(long)]
    request_handover_from: Option<String>,

    /// Safe mode level
    ///
    /// - 0, All features enabled.
    /// - 1, Sync blocks without dispatching messages.
    /// - 2, Sync blocks without storing the trie proofs and dispatching messages.
    ///     In this mode, it is needed to invoke prpc.LoadStorageProof to load the necessary values
    ///     before accepting key handover request.
    #[arg(long)]
    #[arg(default_value_t = 0)]
    safe_mode_level: u8,
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    pal_gramine::print_target_info();

    let sgx = pal_gramine::is_gramine();
    logger::init_subscriber(sgx);
    serve(sgx).await
}

#[tracing::instrument(name = "main", skip_all)]
async fn serve(sgx: bool) -> Result<(), rocket::Error> {
    info!(sgx, "Starting pruntime...");

    let sealing_path;
    let storage_path;
    if sgx {
        // In gramine, the protected files are configured via manifest file. So we must not allow it to
        // be changed at runtime for security reason. Thus hardcoded it to `/data/protected_files` here.
        // Should keep it the same with the manifest config.
        sealing_path = "/data/protected_files";
        storage_path = "/data/storage_files";
    } else {
        sealing_path = "./data/protected_files";
        storage_path = "./data/storage_files";

        fn mkdir(dir: &str) {
            if let Err(err) = std::fs::create_dir_all(dir) {
                panic!("Failed to create {dir}: {err:?}");
            }
        }
        mkdir(sealing_path);
        mkdir(storage_path);
    }

    let args = Args::parse();

    if let Some(address) = &args.address {
        env::set_var("ROCKET_ADDRESS", address);
    }

    if let Some(port) = &args.port {
        env::set_var("ROCKET_PORT", port.to_string());
    }

    let cores: u32 = args.cores.unwrap_or_else(|| num_cpus::get() as _);
    info!(bench_cores = cores);

    let init_args = {
        let args = args.clone();
        InitArgs {
            sealing_path: sealing_path.into(),
            storage_path: storage_path.into(),
            init_bench: args.init_bench,
            version: env!("CARGO_PKG_VERSION").into(),
            git_revision: git_revision().to_string(),
            enable_checkpoint: !args.disable_checkpoint,
            checkpoint_interval: args.checkpoint_interval,
            remove_corrupted_checkpoint: args.remove_corrupted_checkpoint,
            max_checkpoint_files: args.max_checkpoint_files,
            cores,
            public_port: args.public_port,
            safe_mode_level: args.safe_mode_level,
        }
    };
    info!("init_args: {:#?}", init_args);
    if let Some(from) = args.request_handover_from {
        info!(%from, "Starting handover");
        handover::handover_from(&from, init_args)
            .await
            .expect("Handover failed");
        info!("Handover done");
        return Ok(());
    }
    if let Err(err) = runtime::ecall_init(init_args) {
        panic!("Initialize Failed: {err:?}");
    }

    for i in 0..cores {
        thread::Builder::new()
            .name(format!("bench-{i}"))
            .spawn(move || {
                set_thread_idle_policy();
                loop {
                    runtime::ecall_bench_run(i);
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
            })
            .expect("Failed to launch benchmark thread");
    }

    let mut servers = vec![];

    if args.public_port.is_some() {
        let args_clone = args.clone();
        let server_acl = rocket::tokio::spawn(
            async move {
                let _rocket = api_server::rocket_acl(&args_clone)
                    .expect("should not failed as port is provided")
                    .launch()
                    .await
                    .expect("Failed to launch API server");
            }
            .instrument(info_span!("srv-public")),
        );
        servers.push(server_acl);
    }

    let server_internal = rocket::tokio::spawn(
        async move {
            let _rocket = api_server::rocket(&args)
                .launch()
                .await
                .expect("Failed to launch API server");
        }
        .instrument(info_span!("srv-internal")),
    );
    servers.push(server_internal);

    for server in servers {
        server.await.expect("Failed to launch server");
    }

    info!("pRuntime quited");

    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn set_thread_idle_policy() {}

#[cfg(target_os = "linux")]
fn set_thread_idle_policy() {
    let param = libc::sched_param {
        sched_priority: 0,
        #[cfg(target_env = "musl")]
        sched_ss_low_priority: 0,
        #[cfg(target_env = "musl")]
        sched_ss_repl_period: libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        #[cfg(target_env = "musl")]
        sched_ss_init_budget: libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        #[cfg(target_env = "musl")]
        sched_ss_max_repl: 0,
    };

    unsafe {
        let rv = libc::sched_setscheduler(0, libc::SCHED_IDLE, &param);
        if rv != 0 {
            error!("Failed to set thread schedule prolicy to IDLE");
        }
    }
}
