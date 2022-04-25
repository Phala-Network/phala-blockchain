use std::time::Instant;

use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Data, Request, Response};

/// Measuring the time it takes to process a request.
pub struct TimeMeter;

struct StartTime(Instant);

impl Fairing for TimeMeter {
    fn info(&self) -> Info {
        Info {
            name: "Time meter",
            kind: Kind::Request | Kind::Response,
        }
    }

    fn on_request(&self, request: &mut Request, _data: &Data) {
        let _t = request.local_cache(move || StartTime(Instant::now()));
    }

    fn on_response(&self, request: &Request, response: &mut Response) {
        let start_time = request.local_cache(|| StartTime(Instant::now()));
        let cost = start_time.0.elapsed().as_micros().to_string();
        log::info!(target: "measuring", "{} {} cost {} microseconds", request.method(), request.uri(), cost);
        response.set_raw_header("X-Cost-Time", cost);
    }
}
