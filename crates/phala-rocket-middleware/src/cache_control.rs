use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::uri::Origin;
use rocket::{Data, Request, Response};

pub struct StaticCacheControl<MatchFn> {
    match_fn: MatchFn,
}

impl<MF> StaticCacheControl<MF> {
    fn new<MF>(match_fn: MF) -> Self {
        Self { match_fn }
    }
}

#[rocket::async_trait]
impl<MF> Fairing for StaticCacheControl<MF>
where
    MF: Fn(&str) -> bool,
{
    fn info(&self) -> Info {
        Info {
            name: "Cache control",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        if self.match_fn(request.uri().path().as_str()) {
            if response.status().code / 100 == 2 {
                response.set_raw_header("Cache-Control", "max-age=2592000"); // a month
            } else {
                response.set_raw_header("Cache-Control", "no-cache");
            }
        }
    }
}
