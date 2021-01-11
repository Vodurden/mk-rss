use anyhow;
use reqwest::Url;
use std::convert::TryFrom;
use std::str::FromStr;
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

    /// A css selector indicating which HTML node identifies the date this item was published
    pub pub_date_selector: Option<Selector>,

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

impl FromStr for FeedOrder {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(value)
    }
}

impl ToString for FeedOrder {
    fn to_string(&self) -> String {
        match self {
            FeedOrder::Normal => "normal".to_string(),
            FeedOrder::Reversed => "reversed".to_string(),
        }
    }
}
