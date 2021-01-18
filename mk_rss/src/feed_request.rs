use anyhow;
use reqwest::Url;
use std::convert::TryFrom;
use std::cmp;
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


#[derive(Debug, PartialEq)]
pub struct FeedRequestBuilder {
    pub name: String,
    pub url: Url,
    pub item_selector: String,
    pub title_selector: Option<String>,
    pub link_selector: Option<String>,
    pub pub_date_selector: Option<String>,
    pub order: Option<FeedOrder>,
    pub max_items: Option<usize>,
}

impl FeedRequestBuilder {
    pub fn new(name: &str, url: Url, item_selector: &str) -> Self {
        FeedRequestBuilder {
            name: name.to_string(),
            url,
            item_selector: item_selector.to_string(),
            title_selector: None,
            link_selector: None,
            pub_date_selector: None,
            order: None,
            max_items: None
        }
    }

    pub fn maybe_title_selector<S: Into<String>>(&mut self, selector: Option<S>) -> &mut Self {
        self.title_selector = selector.map(|s| s.into());
        self
    }

    pub fn title_selector<S: Into<String>>(&mut self, selector: S) -> &mut Self {
        self.maybe_title_selector(Some(selector))
    }

    pub fn link_selector<S: Into<String>>(&mut self, selector: S) -> &mut Self {
        self.link_selector = Some(selector.into());
        self
    }

    pub fn pub_date_selector<S: Into<String>>(&mut self, selector: S) -> &mut Self {
        self.pub_date_selector = Some(selector.into());
        self
    }

    pub fn order<O: Into<FeedOrder>>(&mut self, order: O) -> &mut Self {
        self.order = Some(order.into());
        self
    }

    pub fn max_items<S: Into<usize>>(&mut self, max_items: S) -> &mut Self {
        self.max_items = Some(max_items.into());
        self
    }

    pub fn build(&self) -> anyhow::Result<FeedRequest> {
        let item_selector = Selector::parse(&self.item_selector)
            .map_err(|e| anyhow::anyhow!("Could not parse item_selector: {:?}", e))?;

        let title_selector = self.title_selector
            .as_ref()
            .map(|s| Selector::parse(s))
            .transpose()
            .map_err(|e| anyhow::anyhow!("Could not parse title_selector: {:?}", e))?;

        let link_selector = self.link_selector
            .as_ref()
            .map(|s| Selector::parse(s))
            .transpose()
            .map_err(|e| anyhow::anyhow!("Could not parse link_selector: {:?}", e))?;

        let pub_date_selector = self.pub_date_selector
            .as_ref()
            .map(|s| Selector::parse(s))
            .transpose()
            .map_err(|e| anyhow::anyhow!("Could not parse pub_date_selector: {:?}", e))?;

        let order = self.order.unwrap_or(FeedOrder::Normal);

        let max_items = self.max_items.unwrap_or(30);
        let max_items = cmp::min(max_items, 30);

        Ok(FeedRequest {
            name: self.name.clone(),
            url: self.url.clone(),
            item_selector,
            title_selector,
            link_selector,
            pub_date_selector,
            order,
            max_items
        })
    }
}


#[derive(Debug, PartialEq, Eq, Copy, Clone)]
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
