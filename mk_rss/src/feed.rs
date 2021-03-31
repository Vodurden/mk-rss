use indoc::formatdoc;
use reqwest::Url;
use chrono::{DateTime, Duration, Local};

use super::website::{Website, WebsiteElement};

#[derive(Debug)]
pub struct Feed {
    pub name: String,
    pub url: Url,
    pub items: Vec<FeedItem>
}

#[derive(Debug)]
pub struct FeedItem {
    pub title: String,
    pub url: Url,
    pub pub_date: DateTime<Local>
}

impl Feed {
    pub fn from_website(website: Website, now: DateTime<Local>) -> Feed {
        let items = Feed::infer_feed_items(website.elements, now);
        let items = Feed::match_pub_dates_to_order(items);

        Feed {
            name: website.name,
            url: website.url,
            items
        }
    }

    pub fn to_rss_xml(&self) -> String {
        let items_xml = self.items
            .iter()
            .map(|item| item.to_rss_xml())
            .collect::<Vec<String>>()
            .join("");

        formatdoc!{"
            <rss version=\"2.0\">
            <channel>

            <title>{}</title>
            <link>{}</link>
            <guid>{}</guid>
            <description/>

            {}

            </channel>
            </rss>
            ",
            self.name,
            self.url,
            self.url,
            items_xml
        }
    }

    fn infer_feed_items(elements: Vec<WebsiteElement>, now: DateTime<Local>) -> Vec<FeedItem> {
        let mut previous_pub_date = elements
            .get(0)
            .and_then(|element| element.pub_date)
            .unwrap_or(now);

        elements
            .into_iter()
            .map(|element| {
                let pub_date = element.pub_date.unwrap_or(previous_pub_date);
                previous_pub_date = pub_date;

                FeedItem {
                    title: element.title,
                    url: element.url,
                    pub_date
                }
            })
            .collect()
    }

    /// We want the `pub_date`'s of `items` to match the order of `items` since
    /// most feeds use `pub_date` to decide what order to show a feed in.
    ///
    /// We also don't want any `pub_date` to be duplicated since the order becomes
    /// undefined. To solve this we jitter the `pub_date` slightly to make the order
    /// match.
    fn match_pub_dates_to_order(items: Vec<FeedItem>) -> Vec<FeedItem> {
        let mut previous_item_pub_date = match items.get(0) {
            Some(first_item) => first_item.pub_date,
            None => return vec![]
        };

        let mut items = items;

        items
            .iter_mut()
            .skip(1)
            .for_each(|item| {
                if item.pub_date >= previous_item_pub_date {
                    item.pub_date = previous_item_pub_date - Duration::seconds(1);
                }

                previous_item_pub_date = item.pub_date;
            });

        items
    }
}

impl FeedItem {
    pub fn to_rss_xml(&self) -> String {
        formatdoc! {"
            <item>
                <title>{}</title>
                <link>{}</link>
                <guid>{}</guid>
                <pubDate>{}</pubDate>
                <description/>
            </item>
            ",
            self.title,
            self.url,
            self.url,
            self.pub_date.to_rfc2822()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::prelude::*;

    fn element_url() -> Url {
        Url::parse("https://example.com/feed/").unwrap()
    }

    #[test]
    pub fn from_website_should_infer_missing_dates_from_previous_date() {
        let website = Website {
            name: "Test Website".into(),
            url: Url::parse("https://example.com/feed/").unwrap(),
            elements: vec![
                website_element("The Story A", None),
                website_element("The Story B", None),
                website_element("The Story C", Some(Local.ymd(2020, 02, 01).and_hms(13, 0, 0))),
                website_element("The Story D", Some(Local.ymd(2020, 02, 01).and_hms(13, 0, 0))),
                website_element("The Story E", Some(Local.ymd(2020, 03, 01).and_hms(13, 0, 0))),
                website_element("The Story F", Some(Local.ymd(2020, 03, 01).and_hms(13, 0, 0))),
            ],
        };

        let now = Local.ymd(2021, 02, 01).and_hms(13, 0, 0);
        let feed = Feed::from_website(website, now);

        for window in feed.items.windows(2) {
            let earlier_item = window.get(0).expect("earlier_item should exist");
            let later_item = window.get(1).expect("later_item should exist");
            assert!(
                earlier_item.pub_date > later_item.pub_date,
                "{} should be published after {}. earlier_pub_date = {}, later_pub_date = {}",
                earlier_item.title,
                later_item.title,
                earlier_item.pub_date,
                later_item.pub_date
            )
        }
    }

    #[test]
    pub fn from_website_should_reorder_pub_date_by_input_order() {
        let website = Website {
            name: "Test Website".into(),
            url: Url::parse("https://example.com/feed/").unwrap(),
            elements: vec![
                website_element("The Story A", Some(Local.ymd(2020, 03, 01).and_hms(13, 0, 0))),
                website_element("The Story B", Some(Local.ymd(2020, 03, 02).and_hms(13, 0, 0))),
                website_element("The Story C", Some(Local.ymd(2020, 02, 01).and_hms(13, 0, 0))),
            ],
        };

        let now = Local.ymd(2021, 02, 01).and_hms(13, 0, 0);
        let feed = Feed::from_website(website, now);

        for window in feed.items.windows(2) {
            let earlier_item = window.get(0).expect("earlier_item should exist");
            let later_item = window.get(1).expect("later_item should exist");
            assert!(
                earlier_item.pub_date > later_item.pub_date,
                "{} should be published after {}. earlier_pub_date = {}, later_pub_date = {}",
                earlier_item.title,
                later_item.title,
                earlier_item.pub_date,
                later_item.pub_date
            )
        }
    }

    fn website_element(title: &str, pub_date: Option<DateTime<Local>>) -> WebsiteElement {
        WebsiteElement {
            title: title.into(),
            url: element_url(),
            pub_date
        }
    }
}
