use env_logger::Logger;

pub use logger::init_env_logger;
pub use subscriber::init_subscriber;

#[cfg(test)]
mod test;

mod logger;
mod subscriber;

fn get_env<T>(name: &str, default: T) -> T
where
    T: std::str::FromStr,
{
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn target_allowed(target: &str) -> bool {
    use MatchMode::*;
    enum MatchMode {
        Prefix,
        Eq,
    }

    // Keep more frequently targets in the front
    let whitelist = [
        ("phactory", Prefix),
        ("rocket::launch", Prefix),
        ("rocket::server", Eq),
        ("pink", Prefix),
        ("sidevm", Prefix),
        ("prpc_measuring", Eq),
        ("gk_computing", Eq),
        ("phala_", Prefix),
        ("pruntime", Prefix),
    ];
    for (rule, mode) in whitelist.into_iter() {
        match mode {
            Prefix => {
                if target.starts_with(rule) {
                    return true;
                }
            }
            Eq => {
                if rule == target {
                    return true;
                }
            }
        }
    }
    false
}
