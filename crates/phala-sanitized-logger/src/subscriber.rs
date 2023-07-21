use std::io::Stderr;
use tracing_core::{span, Event, LevelFilter, Metadata, Subscriber};
use tracing_subscriber::{
    fmt::{
        format::{DefaultFields, Format, Full},
        Subscriber as FmtSubscriber,
    },
    util::SubscriberInitExt,
    EnvFilter,
};

type EnvFilterSubscriber = FmtSubscriber<DefaultFields, Format<Full>, EnvFilter, fn() -> Stderr>;

/// A tracing subscriber that only allow our codes to print logs
struct SanitizedSubscriber(EnvFilterSubscriber);

impl Subscriber for SanitizedSubscriber {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        self.0.enabled(metadata) && super::target_allowed(metadata.target())
    }

    fn new_span(&self, span: &span::Attributes<'_>) -> span::Id {
        self.0.new_span(span)
    }

    fn record(&self, span: &span::Id, values: &span::Record<'_>) {
        self.0.record(span, values)
    }

    fn record_follows_from(&self, span: &span::Id, follows: &span::Id) {
        self.0.record_follows_from(span, follows)
    }

    fn event(&self, event: &Event<'_>) {
        self.0.event(event)
    }

    fn enter(&self, span: &span::Id) {
        self.0.enter(span)
    }

    fn exit(&self, span: &span::Id) {
        self.0.exit(span)
    }

    fn event_enabled(&self, event: &Event<'_>) -> bool {
        self.0.event_enabled(event)
    }

    fn current_span(&self) -> span::Current {
        self.0.current_span()
    }

    fn clone_span(&self, id: &span::Id) -> span::Id {
        self.0.clone_span(id)
    }

    fn try_close(&self, id: span::Id) -> bool {
        self.0.try_close(id)
    }

    fn max_level_hint(&self) -> Option<LevelFilter> {
        self.0.max_level_hint()
    }

    fn register_callsite(&self, metadata: &'static Metadata<'static>) -> tracing_core::Interest {
        self.0.register_callsite(metadata)
    }

    fn drop_span(&self, id: span::Id) {
        #[allow(deprecated)]
        self.0.drop_span(id);
    }
}

pub fn init_subscriber(sanitized: bool) {
    let builder = FmtSubscriber::builder();
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    let ansi = crate::get_env("RUST_LOG_ANSI_COLOR", false);
    let sanitized = crate::get_env("RUST_LOG_SANITIZED", sanitized);
    let builder = builder
        .with_env_filter(filter)
        .with_ansi(ansi)
        .with_writer(std::io::stderr as fn() -> Stderr);
    let subscriber = builder.finish();
    if sanitized {
        SanitizedSubscriber(subscriber)
            .try_init()
            .expect("Failed to init tracing subscriber");
    } else {
        subscriber
            .try_init()
            .expect("Failed to init tracing subscriber");
    }
}
