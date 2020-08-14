use lambda_http::{RequestExt, Request};
use std::convert::TryFrom;

use super::error::Error;

#[derive(Debug, PartialEq, Eq)]
pub struct FeedRequest {
    /// The name of this feed
    name: String,

    /// The uri of this website to scrape.
    ///
    /// We assume this string is a valid URI
    uri: String,

    /// A css selector indicating what part of this sites HTML contains the list
    /// of items to turn into a feed
    ///
    /// See https://jsoup.org/apidocs/org/jsoup/select/Selector.html for pattern details
    item_selector: String,

    /// A css selector indicating which HTML node contains each items title.
    ///
    /// We assume the text of this node contains the title
    title_selector: Option<String>,

    /// A css selector indicating which HTML node contains each items link.
    ///
    /// This must point to an `<a>` tag and we assume the `href` is the target URI
    link_selector: Option<String>,

    /// The order of elements the feed should return.
    ///
    /// `Normal` means the same order as the webpage (top-most item will be considered the "most recent")
    /// `Reversed` is the reverse of `Normal` (bottom-most item will be considered "most recent")
    order: FeedOrder,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FeedOrder { Normal, Reversed }

impl TryFrom<Request> for FeedRequest {
    type Error = Error;

    fn try_from(request: Request) -> Result<Self, Self::Error> {
        let params = request.query_string_parameters();

        let name = params.get("name").ok_or(Error::required_param("name"))?;
        let name = name.to_string();

        let uri = params.get("uri").ok_or(Error::required_param("uri"))?;
        let uri = uri.to_string();

        let item_selector = params
            .get("item_selector")
            .ok_or(Error::required_param("item_selector"))?;
        let item_selector = item_selector.to_string();

        let title_selector = params.get("title_selector");
        let title_selector = title_selector.map(|s| s.to_string());

        let link_selector = params.get("link_selector");
        let link_selector = link_selector.map(|s| s.to_string());

        let order: FeedOrder = match params.get("order") {
            Some(order) => FeedOrder::try_from(order)?,
            None => FeedOrder::Normal,
        };

        Ok(FeedRequest {
            name,
            uri,
            item_selector,
            title_selector,
            link_selector,
            order,
        })
    }
}

impl TryFrom<&str> for FeedOrder {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "normal" => Ok(FeedOrder::Normal),
            "reversed" => Ok(FeedOrder::Reversed),
            _ => Err(Error::InvalidFeedOrder(value.to_string()))
        }
    }

}
