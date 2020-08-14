mod feed_request;
mod error;

use lambda_http::{lambda, IntoResponse, Request, Response};
use lambda_runtime::{Context, error::HandlerError};
use std::convert::TryFrom;

use error::Error;
use feed_request::FeedRequest;

fn main() {
    lambda!(lambda_main)
}

fn lambda_main(
    request: Request,
    ctx: Context
) -> Result<impl IntoResponse, HandlerError> {
    let result = mk_rss(request, ctx);

    let response = result
        .map(|r| r.into_response())
        .unwrap_or_else(|e| {
            Response::builder()
                .status(400)
                .body(format!("{}", e).into())
                .expect("failed to render response")
        });

    Ok(response)
}

fn mk_rss(
    request: Request,
    _ctx: Context
) -> Result<impl IntoResponse, Error> {
    let feed_request = FeedRequest::try_from(request)?;

    Ok(format!("hello {:?}", feed_request))
}
