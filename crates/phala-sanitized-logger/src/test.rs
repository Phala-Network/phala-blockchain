use super::*;

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
            "pink",
            "pink::contract",
            "pink::runtime::extension",
            "pink_extension_runtime",
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

#[test]
fn see_log() {
    use log::info;

    init(true);
    info!(target: "pink", "target pink");
    info!(target: "other", "target other");
}
