mod api_server;
mod pal_gramine;
mod ias;
mod runtime;

use std::{env, thread};

use clap::{AppSettings, Parser};
use log::{error, info};

use phactory::BlockNumber;
use phactory_api::ecall_args::{git_revision, InitArgs};

#[derive(Parser, Debug, Clone)]
#[clap(about = "The Phala TEE worker app.", version, author)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
struct Args {
    /// Number of CPU cores to be used for mining.
    #[clap(short, long)]
    cores: Option<u32>,

    /// Run benchmark at startup.
    #[clap(long)]
    init_bench: bool,

    /// Allow CORS for HTTP
    #[clap(long)]
    allow_cors: bool,

    /// Turn on /kick API
    #[clap(long)]
    enable_kick_api: bool,

    /// Listening IP address of HTTP
    #[clap(long)]
    address: Option<String>,

    /// Listening port of HTTP
    #[clap(long)]
    port: Option<u16>,

    /// Listening port of HTTP (with access control)
    #[clap(long)]
    public_port: Option<u16>,

    /// Disable checkpoint
    #[clap(long)]
    disable_checkpoint: bool,

    /// Checkpoint interval in seconds, default to 5 minutes
    #[clap(long)]
    #[clap(default_value_t = 300)]
    checkpoint_interval: u64,

    /// Remove corrupted checkpoint so that pruntime can restart to continue to load others.
    #[clap(long)]
    remove_corrupted_checkpoint: bool,

    /// Max number of checkpoint files kept
    #[clap(long)]
    #[clap(default_value_t = 5)]
    max_checkpoint_files: u32,

    /// Measuring the time it takes to process each RPC call.
    #[clap(long)]
    measure_rpc_time: bool,

    /// Run the database garbage collection at given interval in blocks
    #[clap(long)]
    #[clap(default_value_t = 100)]
    gc_interval: BlockNumber,
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    // Disable the thread local arena(memory pool) for glibc.
    // See https://github.com/gramineproject/gramine/issues/342#issuecomment-1014475710
    #[cfg(target_env = "gnu")]
    unsafe {
        libc::mallopt(libc::M_ARENA_MAX, 1);
    }

    let running_under_gramine = std::path::Path::new("/dev/attestation/user_report_data").exists();
    let sealing_path;
    let storage_path;
    if running_under_gramine {
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

    let env = env_logger::Env::default().default_filter_or("info");
    env_logger::Builder::from_env(env).format_timestamp_micros().init();

    let cores: u32 = args.cores.unwrap_or_else(|| num_cpus::get() as _);
    info!("Bench cores: {}", cores);

    let init_args = {
        let args = args.clone();
        InitArgs {
            sealing_path: sealing_path.into(),
            storage_path: storage_path.into(),
            init_bench: args.init_bench,
            version: env!("CARGO_PKG_VERSION").into(),
            git_revision: git_revision(),
            enable_checkpoint: !args.disable_checkpoint,
            checkpoint_interval: args.checkpoint_interval,
            remove_corrupted_checkpoint: args.remove_corrupted_checkpoint,
            max_checkpoint_files: args.max_checkpoint_files,
            gc_interval: args.gc_interval,
            cores,
            public_port: args.public_port,
        }
    };
    info!("init_args: {:#?}", init_args);
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
        let server_acl = rocket::tokio::spawn(async move {
            let _rocket = api_server::rocket_acl(&args_clone)
                .expect("should not failed as port is provided")
                .launch()
                .await
                .expect("Failed to launch API server");
        });
        servers.push(server_acl);
    }

    let server_internal = rocket::tokio::spawn(async move {
        let _rocket = api_server::rocket(&args)
            .launch()
            .await
            .expect("Failed to launch API server");
    });
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
