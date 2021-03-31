use reqwest::Url;
use scraper::Html;
use chrono::{DateTime, Local};
use chrono_english::Dialect;

use super::feed_request::{FeedRequest, FeedOrder};

#[derive(Debug, PartialEq)]
pub struct Website {
    pub name: String,
    pub url: Url,
    pub elements: Vec<WebsiteElement>
}

#[derive(Debug, PartialEq, Eq)]
pub struct WebsiteElement {
    pub title: String,
    pub url: Url,
    pub pub_date: Option<DateTime<Local>>
}

impl Website {
    /// Scrape a `Website` from `html_body`
    pub fn scrape(request: &FeedRequest, html_body: &str, now: DateTime<Local>) -> Website {
        let mut items = Website::scrape_items(request, html_body, now);

        if request.order == FeedOrder::Reversed {
            items.reverse();
        }

        let items: Vec<WebsiteElement> = items
            .into_iter()
            .take(request.max_items)
            .collect();

        Website {
            name: request.name.clone(),
            url: request.url.clone(),
            elements: items
        }
    }

    fn scrape_items(request: &FeedRequest, html_body: &str, now: DateTime<Local>) -> Vec<WebsiteElement> {
        let document = Html::parse_document(html_body);

        document
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
                let pub_date = chrono_english::parse_date_string(&pub_date_text, now, Dialect::Uk).ok();

                Some(WebsiteElement { title, url: absolute_url, pub_date })
            })
            .collect()
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

        let now = Local.ymd(2021, 02, 01).and_hms(13, 0, 0);
        let feed = Website::scrape(&request, html_body, now);

        assert_eq!(feed.elements.get(0).map(|i| i.url.to_string()), Some("https://example.com/feed/item-1".to_string()))
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

        let now = Local.ymd(2021, 02, 01).and_hms(13, 0, 0);
        let feed = Website::scrape(&request, html_body, now);

        assert_eq!(
            feed.elements.get(0).and_then(|i| i.pub_date),
            Local.from_local_date(&NaiveDate::from_ymd(2021, 01, 10)).and_hms_opt(0, 0, 0).earliest()
        );
    }
}
