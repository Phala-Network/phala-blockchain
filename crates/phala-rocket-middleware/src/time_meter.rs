use std::time::Instant;

use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Data, Request, Response};

/// Measuring the time it takes to process a request.
pub struct TimeMeter;

struct StartTime(Instant);

#[rocket::async_trait]
impl Fairing for TimeMeter {
    fn info(&self) -> Info {
        Info {
            name: "Time meter",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _data: &mut Data<'_>) {
        let _t = request.local_cache(move || StartTime(Instant::now()));
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        let start_time = request.local_cache(|| StartTime(Instant::now()));
        let cost = start_time.0.elapsed().as_micros().to_string();
        log::info!(
            target: "prpc_measuring",
            "{} {} cost {} microseconds, status: {}",
            request.method(),
            request.uri(),
            cost,
            response.status().code
        );
        response.set_raw_header("X-Cost-Time", cost);
    }
}
