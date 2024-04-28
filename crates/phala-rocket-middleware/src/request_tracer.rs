use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicU64, Ordering};

use rocket::fairing::{Fairing, Info, Kind};
use rocket::request::{FromRequest, Outcome};
use rocket::{Data, Request, Response};

/// Set a unique trace id for each request.
pub struct RequestTracer {
    sn: AtomicU64,
    step: u64,
}

impl Default for RequestTracer {
    fn default() -> Self {
        Self::new(1)
    }
}

impl RequestTracer {
    /// Create a new RequestTracer with a given step.
    pub fn new(step: u64) -> Self {
        Self {
            sn: AtomicU64::new(0),
            step,
        }
    }

    fn next_id(&self) -> TraceId {
        TraceId(self.sn.fetch_add(self.step, Ordering::Relaxed))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TraceId(u64);

impl Display for TraceId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl TraceId {
    pub fn id(&self) -> u64 {
        self.0
    }
}

#[rocket::async_trait]
impl Fairing for RequestTracer {
    fn info(&self) -> Info {
        Info {
            name: "Reqeust Tracer",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _data: &mut Data<'_>) {
        let trace_id = self.next_id();
        let _t = request.local_cache(|| trace_id);
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        let trace = request.local_cache(TraceId::default);
        response.set_raw_header("X-Request-Id", trace.id().to_string());
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for TraceId {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let trace = request.local_cache(TraceId::default);
        Outcome::Success(*trace)
    }
}
