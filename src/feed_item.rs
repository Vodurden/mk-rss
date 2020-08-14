use reqwest::Url;
use scraper::Html;

use super::feed_request::{FeedRequest, FeedOrder};

#[derive(Debug, PartialEq, Eq)]
pub struct FeedItem {
    pub title: String,
    pub url: Url
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

                Some(FeedItem { title, url })
            })
            .collect();

        if request.order == FeedOrder::Reversed {
            items.reverse();
        }

        let items = items.into_iter().take(request.max_items).collect();

        items
    }
}
