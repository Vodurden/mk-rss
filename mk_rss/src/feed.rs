use indoc::formatdoc;
use reqwest::Url;
use scraper::Html;
use chrono::{DateTime, Duration, Local};
use chrono_english::Dialect;

use super::feed_request::{FeedRequest, FeedOrder};

#[derive(Debug, PartialEq)]
pub struct Feed {
    name: String,
    url: Url,
    items: Vec<FeedItem>
}

#[derive(Debug, PartialEq, Eq)]
pub struct FeedItem {
    pub title: String,
    pub url: Url,
    pub pub_date: Option<DateTime<Local>>,
}

impl Feed {
    pub fn scrape(request: &FeedRequest, html_body: &str) -> Feed {
        let document = Html::parse_document(html_body);

        let mut items: Vec<FeedItem> = document
            .select(&request.item_selector)
            .filter_map(|item| {
                let title_node = request.title_selector
                    .as_ref()
                    .and_then(|s| item.select(&s).next())
                    .unwrap_or(item);

                let title = title_node.text().collect::<String>().trim().to_string();
                let link_node = request.link_selector
                    .as_ref()
                    .and_then(|s| item.select(&s).next())
                    .unwrap_or(item);

                let url = link_node.value().attr("href")?.to_string();
                let absolute_url = Url::parse(&url)
                    .ok()
                    .or_else(|| request.url.join(&url).ok())?;

                let pub_date_node = request.pub_date_selector
                    .as_ref()
                    .and_then(|s| item.select(&s).next())
                    .unwrap_or(item);

                let pub_date_text = pub_date_node.text().collect::<String>().trim().to_string();
                let pub_date = chrono_english::parse_date_string(&pub_date_text, Local::now(), Dialect::Uk).ok();

                Some(FeedItem { title, url: absolute_url, pub_date })
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
        let mut item_pub_date = Local::now();
        for item in items.iter_mut() {
            item.pub_date = Some(item.pub_date.unwrap_or(item_pub_date));
            item_pub_date = item_pub_date - Duration::hours(1);
        }

        Feed {
            name: request.name.clone(),
            url: request.url.clone(),
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
}

impl FeedItem {
    pub fn to_rss_xml(&self) -> String {
        let pub_date_string = self.pub_date
            .map(|d| {
                format!("<pubDate>{}</pubDate>", d.to_rfc2822())
            })
            .unwrap_or_else(|| String::new());

        formatdoc! {"
            <item>
                <title>{}</title>
                <link>{}</link>
                <guid>{}</guid>
                {}
                <description/>
            </item>
            ",
            self.title,
            self.url,
            self.url,
            pub_date_string
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Selector;
    use indoc::indoc;
    use chrono::{TimeZone, NaiveDate};

    /// When parsing items from HTML we need to deal with two types of links:
    ///
    /// - Relative: `<a href="/a/sub/page.html" />`
    /// - Absolute: `<a href="www.example.com/a/sub/page.html />`
    ///
    /// Our RSS output needs all links to be absolute, so we want to test that the rss scraping
    /// correctly transforms relative links into absolute links.
    #[test]
    pub fn parse_relative_links() {
        let request = FeedRequest {
            name: "Parse Relative RSS Links Test".into(),
            url: Url::parse("https://example.com/feed/").unwrap(),
            item_selector: Selector::parse(".item").unwrap(),
            title_selector: None,
            link_selector: None,
            pub_date_selector: None,
            order: FeedOrder::Normal,
            max_items: 30,
        };

        let html_body = indoc! {r#"
            <!DOCTYPE html>
            <html lang="en-US">
            <body>
                <a class="item" href="item-1">Item 1</a>
                <a class="item" href="item-2">Item 2</a>
            </body>
        "#};

        let feed = Feed::scrape(&request, html_body);

        assert_eq!(feed.items.get(0).map(|i| i.url.to_string()), Some("https://example.com/feed/item-1".to_string()))
    }

    #[test]
    pub fn parse_human_dates() {
        let request = FeedRequest {
            name: "Parse Human Dates Test".into(),
            url: Url::parse("https://example.com/feed/").unwrap(),
            item_selector: Selector::parse(".item").unwrap(),
            title_selector: Selector::parse(".link").ok(),
            link_selector: Selector::parse(".link").ok(),
            pub_date_selector: Selector::parse(".published").ok(),
            order: FeedOrder::Normal,
            max_items: 30,
        };

        let html_body = indoc! {r#"
            <!DOCTYPE html>
            <html lang="en-US">
            <body>
              <div class="item">
                <a class="link" href="item-1">The Story</a>
                <p class="published">Jan 10, 2021</p>
              </div>
            </body>
        "#};

        let feed = Feed::scrape(&request, html_body);

        assert_eq!(
            feed.items.get(0).and_then(|i| i.pub_date),
            Local.from_local_date(&NaiveDate::from_ymd(2021, 01, 10)).and_hms_opt(0, 0, 0).earliest()
        );
    }
}
