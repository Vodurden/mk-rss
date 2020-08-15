use indoc::formatdoc;

use super::feed_request::FeedRequest;
use super::feed_item::FeedItem;

pub fn xml_string(request: FeedRequest, items: Vec<FeedItem>) -> String {
    let items_xml = items
        .into_iter()
        .map(item_xml_string)
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
        request.name,
        request.url,
        request.url,
        items_xml
    }
}

fn item_xml_string(item: FeedItem) -> String {
    let pub_date_string = item.pub_date
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
        item.title,
        item.url,
        item.url,
        pub_date_string
    }
}
