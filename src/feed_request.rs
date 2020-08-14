use anyhow::{self, Context};
use lambda_http::{RequestExt, Request};
use reqwest::Url;
use std::convert::TryFrom;
use std::cmp;
use scraper::Selector;

#[derive(Debug, PartialEq)]
pub struct FeedRequest {
    /// The name of this feed
    pub name: String,

    /// The url of this website to scrape.
    pub url: Url,

    /// A css selector indicating what part of this sites HTML contains the list
    /// of items to turn into a feed
    ///
    /// See https://jsoup.org/apidocs/org/jsoup/select/Selector.html for pattern details
    pub item_selector: Selector,

    /// A css selector indicating which HTML node contains each items title.
    ///
    /// We assume the text of this node contains the title
    pub title_selector: Option<Selector>,

    /// A css selector indicating which HTML node contains each items link.
    ///
    /// This must point to an `<a>` tag and we assume the `href` is the target URL
    pub link_selector: Option<Selector>,

    /// The order of elements the feed should return.
    ///
    /// `Normal` means the same order as the webpage (top-most item will be considered the "most recent")
    /// `Reversed` is the reverse of `Normal` (bottom-most item will be considered "most recent")
    pub order: FeedOrder,

    /// The maximum number of items to return
    pub max_items: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FeedOrder { Normal, Reversed }

impl TryFrom<Request> for FeedRequest {
    type Error = anyhow::Error;

    fn try_from(request: Request) -> anyhow::Result<Self> {
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
            order,
            max_items
        })
    }
}

impl TryFrom<&str> for FeedOrder {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "normal" => Ok(FeedOrder::Normal),
            "reversed" => Ok(FeedOrder::Reversed),
            _ => Err(anyhow::anyhow!("{} is not a valid order (valid orders are 'normal' and 'reversed')"))
        }
    }

}
