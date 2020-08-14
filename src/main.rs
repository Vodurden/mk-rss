use lambda_http::{lambda, IntoResponse, Request, RequestExt};
use lambda_runtime::{Context, error::HandlerError};

fn main() {
    lambda!(mk_rss)
}

fn mk_rss(
    request: Request,
    _ctx: Context
) -> Result<impl IntoResponse, HandlerError> {
    Ok(format!(
        "hello {}",
        request
            .query_string_parameters()
            .get("name")
            .unwrap_or_else(|| "stranger")
    ))
}
