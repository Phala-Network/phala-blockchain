use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::Request;

pub struct Token {
    pub value: String,
}

pub struct Authorized;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Authorized {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let token = request
            .rocket()
            .state::<Token>()
            .expect("Token state not available.");
        if token.value.is_empty() {
            return request::Outcome::Success(Authorized);
        }
        match request.headers().get_one("X-Token") {
            Some(header_token) if header_token == token.value => {
                request::Outcome::Success(Authorized)
            }
            _ => request::Outcome::Failure((Status::Forbidden, ())),
        }
    }
}
