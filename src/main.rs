mod feed_request;
mod feed_item;
mod fetch;
mod rss;

use anyhow;
use lambda_http::{IntoResponse, Request, Response};
use lambda_http::lambda;
use std::convert::TryFrom;

use feed_request::FeedRequest;
use feed_item::FeedItem;
use fetch::fetch_url;

#[lambda::lambda(http)]
#[tokio::main]
async fn main(request: Request, _: lambda::Context) -> Result<impl IntoResponse, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let response = match mk_rss(request).await {
        Ok(response) => response.into_response(),
        Err(e) => {
            Response::builder()
                .status(400)
                .body(format!("{}", e).into())
                .expect("failed to render response")
        }
    };

    Ok(response)
}

async fn mk_rss(request: Request) -> anyhow::Result<impl IntoResponse> {
    let request = FeedRequest::try_from(request)?;

    let body = fetch_url(request.url.clone()).await?;

    let items = FeedItem::scrape_all(&request, &body);

    let xml = rss::xml_string(request, items);

    let response = Response::builder()
        .status(200)
        .header("Content-Type", "application/rss+xml")
        .body(xml)
        .expect("failed to render response");

    Ok(response)
}
