use super::*;
use rusty_fork::rusty_fork_test;

#[test]
fn whitelist_works() {
    let allowed: Vec<_> = include_str!("all-log-targets.txt")
        .split('\n')
        .filter(|t| target_allowed(t))
        .collect();
    assert_eq!(
        allowed,
        [
            "gk_computing",
            "phactory",
            "phactory::benchmark",
            "phactory::bin_api_service",
            "phactory::contracts::pink",
            "phactory::contracts::support",
            "phactory::contracts::support::keeper",
            "phactory::light_validation",
            "phactory::light_validation::justification::communication",
            "phactory::prpc_service",
            "phactory::storage::storage_ext",
            "phactory::system",
            "phactory::system::gk",
            "phactory::system::master_key",
            "phactory_api::storage_sync",
            "phala_mq",
            "phala_node_runtime",
            "phala_pallets::mining::pallet::migrations",
            "pink",
            "pink::contract",
            "pink::runtime::extension",
            "pink_chain_extension",
            "prpc_measuring",
            "pruntime",
            "pruntime::api_server",
            "pruntime::ias",
            "pruntime::pal_gramine",
            "pruntime::runtime",
            "rocket::launch",
            "rocket::launch_",
            "rocket::server",
            "sidevm",
            "sidevm_env::tasks",
            "sidevm_host_runtime::instrument",
            "sidevm_host_runtime::resource",
        ]
    );
}

rusty_fork_test! {
    #[test]
    fn show_log_via_env_logger() {
        use log::info;

        std::env::set_var("RUST_LOG_SANITIZED", "1");

        init_env_logger(true);
        info!(target: "pink", "target pink");
        info!(target: "other", "target other");
    }

    #[test]
    fn show_log_via_tracing_subsriber() {
        use log::info;

        std::env::set_var("RUST_LOG_SANITIZED", "1");

        init_subscriber(true);
        info!(target: "pink", "target pink");
        info!(target: "other", "target other");
    }

    #[test]
    fn show_log_via_env_logger_no_sanitized() {
        init_env_logger(false);
        log::info!(target: "pink", "target pink");
    }

    #[test]
    fn show_log_via_tracing_no_sanitized() {
        init_subscriber(false);
        log::info!(target: "pink", "target pink");
    }
}
