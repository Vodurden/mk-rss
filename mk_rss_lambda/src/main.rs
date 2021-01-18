use anyhow::{self, Context};
use netlify_lambda_http::{IntoResponse, Request, RequestExt, Response};
use netlify_lambda_http::lambda;
use std::convert::TryFrom;
use reqwest::Url;

use mk_rss::{self, FeedRequest, FeedRequestBuilder, FeedOrder};

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

    let get_required = |name: &str| -> anyhow::Result<String> {
        params
            .get(name)
            .ok_or(anyhow::anyhow!("{} is required", name))
            .map(|s| s.to_string())
    };

    let name = get_required("name")?;
    let url = get_required("url").and_then(|s| Url::parse(&s).context("Could not parse URL"))?;
    let item_selector = get_required("item_selector")?;

    let max_items = params
        .get("max_items")
        .map(|s| s.parse::<usize>().context("max_items must be a number"))
        .transpose()?;

    let order = params
        .get("order")
        .map(|s| FeedOrder::try_from(s))
        .transpose()?;

    let feed_request_builder = FeedRequestBuilder {
        name,
        url,
        item_selector,
        title_selector: params.get("title_selector").map(|s| s.to_string()),
        link_selector: params.get("link_selector").map(|s| s.to_string()),
        pub_date_selector: params.get("pub_date_selector").map(|s| s.to_string()),
        max_items,
        order
    };

    let feed_request = feed_request_builder.build()?;

    Ok(feed_request)
}


#[cfg(test)]
mod tests {
    use super::*;
    use mk_rss::FeedRequest;
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

        let expected = FeedRequestBuilder::new("Example RSS", Url::parse("https://example.com/feed").unwrap(), ".class")
            .title_selector(".title-class")
            .link_selector(".link-class")
            .pub_date_selector(".pub-date-class")
            .order(FeedOrder::Reversed)
            .max_items(25 as usize)
            .build()
            .unwrap();

        let expected: anyhow::Result<FeedRequest, String> = Ok(expected);

        let feed_request = make_feed_request(request)
            .map_err(|e| format!("{}", e));

        assert_eq!(feed_request, expected);
    }
}
