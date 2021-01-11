use anyhow::{self, Context};
use netlify_lambda_http::{IntoResponse, Request, RequestExt, Response};
use netlify_lambda_http::lambda;
use std::convert::TryFrom;
use std::cmp;
use reqwest::Url;
use scraper::Selector;

use mk_rss::{self, FeedRequest, FeedOrder};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[lambda::lambda(http)]
#[tokio::main]
async fn main(request: Request, _: lambda::Context) -> Result<impl IntoResponse, Error> {
    let feed_request = make_feed_request(request)?;
    let response = match mk_rss::fetch_feed(feed_request).await {
        Ok(feed) => {
            let xml = feed.to_rss_xml();

            Response::builder()
                .status(200)
                .header("Content-Type", "application/rss+xml")
                .body(xml)
                .expect("failed to render response")
        },

        Err(e) => {
            Response::builder()
                .status(400)
                .body(format!("{}", e).into())
                .expect("failed to render response")
        }
    };

    Ok(response)
}

fn make_feed_request(request: Request) -> anyhow::Result<FeedRequest> {
    let params = request.query_string_parameters();

    let name = params.get("name")
        .ok_or(anyhow::anyhow!("name is required"))?
        .to_string();

    let url: Url = params.get("url")
        .ok_or(anyhow::anyhow!("url is required"))
        .and_then({|url|
            Url::parse(url).context("invalid url")
        })?;

    let get_selector = |name: &str| -> anyhow::Result<Option<Selector>> {
        params
            .get(name)
            .map(Selector::parse)
            .map(|s| {
                s.map_err(|_| anyhow::anyhow!("invalid {}", name))
            })
            .transpose()
    };

    let item_selector = get_selector("item_selector")?
        .ok_or(anyhow::anyhow!("item_selector is required"))?;

    let title_selector = get_selector("title_selector")?;
    let link_selector = get_selector("link_selector")?;
    let pub_date_selector = get_selector("pub_date_selector")?;

    let order: FeedOrder = match params.get("order") {
        Some(order) => FeedOrder::try_from(order)?,
        None => FeedOrder::Normal,
    };

    let max_items = params
        .get("max_items")
        .map(|s| {
            s.parse::<usize>().context("max_items must be a number")
        })
        .transpose()?
        .unwrap_or(30);
    let max_items = cmp::min(max_items, 30);

    Ok(FeedRequest {
        name,
        url,
        item_selector,
        title_selector,
        link_selector,
        pub_date_selector,
        order,
        max_items
    })
}


#[cfg(test)]
mod tests {
    use super::*;
    use netlify_lambda_http::{Request, RequestExt};
    use itertools::Itertools;

    #[test]
    pub fn parse_valid_request() {
        let params = vec![
            ("name", "Example RSS"),
            ("url", "https://example.com/feed"),
            ("item_selector", ".class"),
            ("title_selector", ".title-class"),
            ("link_selector", ".link-class"),
            ("pub_date_selector", ".pub-date-class"),
            ("order", "reversed"),
            ("max_items", "25"),
        ];

        let params = params
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .into_group_map();

        let request = Request::default()
            .with_query_string_parameters(params);

        let expected = FeedRequest {
            name: "Example RSS".into(),
            url: Url::parse("https://example.com/feed").unwrap(),
            item_selector: Selector::parse(".class").unwrap(),
            title_selector: Some(Selector::parse(".title-class").unwrap()),
            link_selector: Some(Selector::parse(".link-class").unwrap()),
            pub_date_selector: Some(Selector::parse(".pub-date-class").unwrap()),
            order: FeedOrder::Reversed,
            max_items: 25
        };
        let expected: anyhow::Result<FeedRequest, String> = Ok(expected);

        let feed_request = make_feed_request(request)
            .map_err(|e| format!("{}", e));

        assert_eq!(feed_request, expected);
    }
}
