use super::*;
/// A logger that only allow our codes to print logs
struct SanitizedLogger(Logger);

pub fn init_env_logger(sanitized: bool) {
    let sanitized = crate::get_env("RUST_LOG_SANITIZED", sanitized);
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
