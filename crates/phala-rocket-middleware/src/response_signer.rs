use std::io::Cursor;

use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Request, Response};

/// A signer to sign the response with given signing fn.
pub struct ResponseSigner<SignFn> {
    max_signing_body_size: usize,
    sign_fn: SignFn,
}

impl<SignFn> ResponseSigner<SignFn> {
    pub fn new(max_signing_body_size: usize, sign_fn: SignFn) -> Self {
        Self {
            max_signing_body_size,
            sign_fn,
        }
    }
}

#[rocket::async_trait]
impl<SignFn> Fairing for ResponseSigner<SignFn>
where
    SignFn: Fn(&[u8]) -> Option<String> + Send + Sync + 'static,
{
    fn info(&self) -> Info {
        Info {
            name: "ResponseSigner",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        // TODO.kevin: use `let else` instead when the next rustc released
        let body_size = response.body().preset_size();
        let body = match body_size {
            Some(body_size) if body_size <= self.max_signing_body_size => {
                let mut body = response.body_mut().take();
                if let Ok(body) = body.to_bytes().await {
                    body
                } else {
                    return;
                }
            }
            _ => return,
        };
        if let Some(signature) = (self.sign_fn)(&body) {
            response.set_raw_header("X-Phactory-Signature", signature);
        }
        response.set_sized_body(body.len(), Cursor::new(body));
    }
}
