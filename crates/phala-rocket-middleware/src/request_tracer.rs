use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicU64, Ordering};

use rocket::fairing::{Fairing, Info, Kind};
use rocket::request::{FromRequest, Outcome};
use rocket::{Data, Request, Response};

/// Set a unique trace id for each request.
pub struct RequestTracer;

#[derive(Debug, Clone, Copy)]
pub struct TraceId(u64);

impl Display for TraceId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl TraceId {
    fn next() -> Self {
        static NEXT_TRACE_ID: AtomicU64 = AtomicU64::new(0);
        TraceId(NEXT_TRACE_ID.fetch_add(1, Ordering::SeqCst))
    }

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
        let _t = request.local_cache(|| TraceId::next());
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        let trace = request.local_cache(|| TraceId::next());
        response.set_raw_header("X-Request-Id", trace.id().to_string());
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for TraceId {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let trace = request.local_cache(|| TraceId::next());
        Outcome::Success(*trace)
    }
}
