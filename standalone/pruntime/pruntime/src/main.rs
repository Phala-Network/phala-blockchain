#![feature(decl_macro)]

mod api_server;
mod pal_gramine;
mod ias;
mod runtime;

use std::{env, thread};

use log::{error, info};
use structopt::StructOpt;

use phactory_api::ecall_args::{git_revision, InitArgs};

#[derive(StructOpt, Debug)]
#[structopt(name = "pruntime", about = "The Phala TEE worker app.")]
struct Args {
    /// Number of CPU cores to be used for mining.
    #[structopt(short, long)]
    cores: Option<u32>,

    /// Run benchmark at startup.
    #[structopt(long)]
    init_bench: bool,

    #[structopt(long, default_value = "./GeoLite2-City.mmdb")]
    geoip_city_db: String,

    /// Allow CORS for HTTP
    #[structopt(long)]
    allow_cors: bool,

    /// Turn on /kick API
    #[structopt(long)]
    enable_kick_api: bool,

    /// Log filter passed to env_logger
    #[structopt(long, default_value = "INFO")]
    log_filter: String,

    /// Listening IP address of HTTP
    #[structopt(long)]
    address: Option<String>,

    /// Listening port of HTTP
    #[structopt(long)]
    port: Option<String>,
}

fn main() {
    // Disable the thread local arena(memory pool) for glibc.
    // See https://github.com/gramineproject/gramine/issues/342#issuecomment-1014475710
    #[cfg(target_env = "gnu")]
    unsafe {
        libc::mallopt(libc::M_ARENA_MAX, 1);
    }

    let runing_under_gramine = std::path::Path::new("/dev/attestation/user_report_data").exists();
    let sealing_path = if runing_under_gramine {
        // In gramine, the protected files are configured via manifest file. So we must not allow it to
        // be changed at runtime for security reason. Thus hardcoded it to `/protected_files` here.
        // Should keep it the same with the manifest config.
        "/protected_files"
    } else {
        "./data"
    }
    .into();

    let args = Args::from_args();

    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("ROCKET_ENV", "dev");

    if let Some(address) = &args.address {
        env::set_var("ROCKET_ADDRESS", address);
    }

    if let Some(port) = &args.port {
        env::set_var("ROCKET_PORT", port);
    }

    let env = env_logger::Env::default().default_filter_or(&args.log_filter);
    env_logger::Builder::from_env(env).init();

    let init_args = InitArgs {
        sealing_path,
        log_filter: Default::default(),
        init_bench: args.init_bench,
        version: env!("CARGO_PKG_VERSION").into(),
        git_revision: git_revision(),
        geoip_city_db: args.geoip_city_db,
        enable_checkpoint: false,
        checkpoint_interval: 0,
        skip_corrupted_checkpoint: false,
    };
    info!("init_args: {:#?}", init_args);
    if let Err(err) = runtime::ecall_init(init_args) {
        panic!("Initialize Failed: {:?}", err);
    }

    let bench_cores: u32 = args.cores.unwrap_or_else(|| num_cpus::get() as _);
    info!("Bench cores: {}", bench_cores);

    let rocket = thread::Builder::new()
        .name("rocket".into())
        .spawn(move || {
            let err = api_server::rocket(args.allow_cors, args.enable_kick_api).launch();
            panic!("Launch rocket failed: {}", err);
        })
        .expect("Failed to launch Rocket");

    let mut v = vec![];
    for i in 0..bench_cores {
        let child = thread::Builder::new()
            .name(format!("bench-{}", i))
            .spawn(move || {
                set_thread_idle_policy();
                loop {
                    runtime::ecall_bench_run(i);
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
            })
            .expect("Failed to launch benchmark thread");
        v.push(child);
    }

    let _ = rocket.join();
    for child in v {
        let _ = child.join();
    }
    info!("pRuntime quited");
}

fn set_thread_idle_policy() {
    let param = libc::sched_param {
        sched_priority: 0,
        #[cfg(any(target_env = "musl", target_os = "emscripten"))]
        sched_ss_low_priority: 0,
        #[cfg(any(target_env = "musl", target_os = "emscripten"))]
        sched_ss_repl_period: libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        #[cfg(any(target_env = "musl", target_os = "emscripten"))]
        sched_ss_init_budget: libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        #[cfg(any(target_env = "musl", target_os = "emscripten"))]
        sched_ss_max_repl: 0,
    };

    unsafe {
        let rv = libc::sched_setscheduler(0, libc::SCHED_IDLE, &param);
        if rv != 0 {
            error!("Failed to set thread schedule prolicy to IDLE");
        }
    }
}
