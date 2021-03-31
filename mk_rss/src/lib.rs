mod feed_request;
mod feed;
mod website;
mod fetch;

pub use feed::{Feed, FeedItem};
pub use feed_request::{FeedRequestBuilder, FeedRequest, FeedOrder};
pub use website::{Website, WebsiteElement};
use fetch::fetch_url;
use chrono::Local;

pub async fn fetch_feed(request: FeedRequest) -> anyhow::Result<Feed> {
    let body = fetch_url(request.url.clone()).await?;

    let now = Local::now();
    let website = Website::scrape(&request, &body, now);
    let feed = Feed::from_website(website, now);
    Ok(feed)
}
