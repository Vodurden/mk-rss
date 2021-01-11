use clap::Clap;
use reqwest::Url;
use scraper::Selector;
use tokio;

use mk_rss::{self, FeedRequest, FeedOrder};

#[derive(Clap, Debug)]
#[clap(version = "1.0.1", author = "Jake Woods <jake@jakewoods.net>")]
struct Args {
    /// The name of this feed
    #[clap(long)]
    name: String,

    /// The URL of the page to scrape for this feed
    #[clap(long)]
    url: Url,

    /// A jQuery style css selector targeting the HTML nodes that represent a single item in the feed
    #[clap(long)]
    item_selector: String,

    /// A jQuery style css selector targeting the HTML node that contains the title.
    ///
    /// This selector searches within the node indicated by `--item-selector`.
    #[clap(long)]
    title_selector: Option<String>,

    /// A jQuery style css selector indicating the HTML node that contains the url.
    ///
    /// The targeted node _must_ contain a `href` attribute.
    ///
    /// This selector searches within the node indicated by `--item-selector`.
    #[clap(long)]
    link_selector: Option<String>,

    /// A jQuery style css selector indicating the HTML node that contains a human-readable publish date.
    ///
    /// This selector searches within the node indicated by `--item-selector`.
    #[clap(long)]
    pub_date_selector: Option<String>,

    /// The order of items to return.
    ///
    /// "normal" returns the items in the order they appear on the page from top to bottom.
    /// "reversed" returns hte items in the reverse order
    #[clap(long, default_value = "normal")]
    order: FeedOrder,

    /// The maximum number of items to return in the feed.
    #[clap(long, default_value = "30")]
    max_items: u32,

    #[clap(subcommand)]
    command: Command
}

#[derive(Clap, Debug)]
enum Command {
    /// Fetch the indicated feed and return the generate RSS XML on standard output.
    #[clap()]
    Fetch,

    /// Convert the arguments of this command into URL parameters suitable for querying the lambda endpoint of mk-rss
    #[clap()]
    ToRSSUrl(ToRSSUrl)
}

#[derive(Clap, Debug)]
struct ToRSSUrl {
    /// The URL currently hosting the mk-rss lambda.
    #[clap(long)]
    lambda_url: Url,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();

    match args.command {
        Command::Fetch => fetch(args).await?,
        Command::ToRSSUrl(ref command_args) => to_rss_url(&args, command_args),
    };

    Ok(())
}

async fn fetch(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let feed_request = FeedRequest {
        name: args.name,
        url: args.url,
        item_selector: Selector::parse(&args.item_selector).unwrap(),
        title_selector: args.title_selector.and_then(|t| Selector::parse(&t).ok()),
        link_selector: args.link_selector.and_then(|l| Selector::parse(&l).ok()),
        pub_date_selector: args.pub_date_selector.and_then(|pd| Selector::parse(&pd).ok()),
        order: FeedOrder::Normal,
        max_items: 30
    };

    let feed = mk_rss::fetch_feed(feed_request).await?;
    println!("{}", feed.to_rss_xml());

    Ok(())
}

fn to_rss_url(args: &Args, command_args: &ToRSSUrl) {
    let mut rss_url = command_args.lambda_url.clone();

    rss_url
        .query_pairs_mut()
        .append_pair("name", &args.name)
        .append_pair("url", &args.url.as_str())
        .append_pair("item_selector", &args.item_selector);

    if let Some(title_selector) = &args.title_selector {
        rss_url.query_pairs_mut()
               .append_pair("title_selector", title_selector);
    }

    if let Some(link_selector) = &args.link_selector {
        rss_url.query_pairs_mut()
               .append_pair("link_selector", link_selector);
    }

    if let Some(pub_date_selector) = &args.pub_date_selector {
        rss_url.query_pairs_mut()
               .append_pair("pub_date_selector", pub_date_selector);
    }

    rss_url
        .query_pairs_mut()
        .append_pair("order", &args.order.to_string())
        .append_pair("max_items", &args.max_items.to_string());

    println!("{}", rss_url);
}
