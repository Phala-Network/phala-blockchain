use env_logger::Logger;

#[cfg(test)]
mod test;

/// A logger that only allow our codes to print logs
struct SanitizedLogger(Logger);

pub(crate) fn init(sanitized: bool) {
    let env = env_logger::Env::default().default_filter_or("info");
    let mut builder = env_logger::Builder::from_env(env);
    builder.format_timestamp_micros();
    if sanitized {
        let env_logger = builder.build();
        let max_level = env_logger.filter();
        let logger = SanitizedLogger(env_logger);
        log::set_boxed_logger(Box::new(logger)).expect("Failed to install sanitized logger");
        log::set_max_level(max_level);
    } else {
        builder.init();
    }
}

impl log::Log for SanitizedLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.0.enabled(metadata) && target_allowed(metadata.target())
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            self.0.log(record)
        }
    }

    fn flush(&self) {
        self.0.flush()
    }
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
        ("phala_mq", Eq),
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
