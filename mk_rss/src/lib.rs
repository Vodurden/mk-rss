mod feed_request;
mod feed;
mod fetch;

pub use feed::{Feed, FeedItem};
pub use feed_request::{FeedRequest, FeedOrder};
use fetch::fetch_url;

pub async fn fetch_feed(request: FeedRequest) -> anyhow::Result<Feed> {
    let body = fetch_url(request.url.clone()).await?;
    Ok(Feed::scrape(&request, &body))
}
