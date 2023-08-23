use actix_web::error::ErrorBadRequest;
use actix_web::{dev, Error, FromRequest, HttpRequest};
use futures_util::future::{err, ok, Ready};
use std::convert::From;
use woothee::parser::{Parser, WootheeResult};
use woothee::woothee::VALUE_UNKNOWN;

#[derive(Debug)]
pub struct UserAgent {
    pub name: Option<String>,
    pub category: Option<String>,
    pub os: Option<String>,
}

impl From<WootheeResult<'_>> for UserAgent {
    fn from(result: WootheeResult) -> Self {
        // TODO : in future when then_some method will be stable remake this part
        // to refactor this part
        Self {
            name: {
                let name = result.name.to_string();

                (name != VALUE_UNKNOWN).then(|| name)
            },
            category: {
                let category = result.category.to_string();

                (category != VALUE_UNKNOWN).then(|| category)
            },
            os: {
                let os = result.os.to_string();

                (os != VALUE_UNKNOWN).then(|| os)
            },
        }
    }
}

impl FromRequest for UserAgent {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _: &mut dev::Payload) -> Self::Future {
        if let Some(ua) = req.headers().get("User-Agent") {
            let parser = Parser::new();

            if let Some(result) = parser.parse(ua.to_str().unwrap()) {
                return ok(UserAgent::from(result));
            }
        }

        err(ErrorBadRequest("no luck"))
    }
}
