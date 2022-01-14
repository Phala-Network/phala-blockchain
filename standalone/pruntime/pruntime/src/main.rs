#![feature(decl_macro)]

mod api_server;
mod pal_gramine;
mod runtime;
mod ra;

use std::{env, path, thread};

use log::{error, info};
use serde::Deserialize;
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

    /// Disable checkpoint
    #[structopt(long)]
    disable_checkpoint: bool,

    /// Checkpoint interval in seconds, default to 5 minutes
    #[structopt(long)]
    #[structopt(default_value = "300")]
    checkpoint_interval: u64,
}

#[derive(Deserialize, Debug)]
struct Env {
    #[serde(default = "default_state_file_path")]
    state_file_path: String,

    #[serde(default)]
    allow_cors: bool,

    #[serde(default)]
    enable_kick_api: bool,
}

fn default_state_file_path() -> String {
    "./".into()
}

fn main() {
    let args = Args::from_args();

    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("ROCKET_ENV", "dev");

    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let env = envy::from_env::<Env>().unwrap();

    let executable = env::current_exe().unwrap();
    let path = executable.parent().unwrap();
    let sealing_path: path::PathBuf = path.join(&env.state_file_path);
    let sealing_path = String::from(sealing_path.to_str().unwrap());
    let init_args = InitArgs {
        sealing_path,
        log_filter: Default::default(),
        init_bench: args.init_bench,
        version: env!("CARGO_PKG_VERSION").into(),
        git_revision: git_revision(),
        geoip_city_db: args.geoip_city_db,
        enable_checkpoint: !args.disable_checkpoint,
        checkpoint_interval: args.checkpoint_interval,
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
            let err = api_server::rocket(env.allow_cors, env.enable_kick_api).launch();
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
    let param = libc::sched_param { sched_priority: 0 };
    unsafe {
        let rv = libc::sched_setscheduler(0, libc::SCHED_IDLE, &param);
        if rv != 0 {
            error!("Failed to set thread schedule prolicy to IDLE");
        }
    }
}
