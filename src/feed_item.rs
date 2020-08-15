use reqwest::Url;
use scraper::Html;
use chrono::{DateTime, Utc, Duration};

use super::feed_request::{FeedRequest, FeedOrder};

#[derive(Debug, PartialEq, Eq)]
pub struct FeedItem {
    pub title: String,
    pub url: Url,
    pub pub_date: Option<DateTime<Utc>>,
}

impl FeedItem {
    pub fn scrape_all(request: &FeedRequest, body: &str) -> Vec<FeedItem> {
        let document = Html::parse_document(body);

        let mut items: Vec<FeedItem> = document
            .select(&request.item_selector)
            .filter_map(|item| {
                let title_node = request.title_selector
                    .clone()
                    .and_then(|s| item.select(&s).next())
                    .unwrap_or(item);

                let title = title_node.text().collect::<String>();

                let link_node = request.link_selector
                    .clone()
                    .and_then(|s| item.select(&s).next())
                    .unwrap_or(item);

                let url = link_node.value().attr("href")?.to_string();
                let url = request.url.join(&url).ok()?;

                Some(FeedItem { title, url, pub_date: None })
            })
            .collect();

        if request.order == FeedOrder::Reversed {
            items.reverse();
        }

        let mut items: Vec<FeedItem> = items
            .into_iter()
            .take(request.max_items)
            .collect();

        // From a lot of sites we don't have a good way to get the
        // publication date. Instead we synthesise a date to keep
        // the feed order consistent.
        let mut item_pub_date = Utc::now();
        for item in items.iter_mut() {
            item.pub_date = Some(item_pub_date);
            item_pub_date = item_pub_date - Duration::hours(1);
        }

        items
    }
}
