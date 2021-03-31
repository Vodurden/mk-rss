use anyhow::Context;
use reqwest::Url;
use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::hash::{Hash, Hasher};
use std::time::{Duration, SystemTime};

pub async fn fetch_url(url: Url) -> anyhow::Result<String> {
    let cache_path = calculate_cache_path(&url);

    // Currently let's cache for 30 minutes and see how things go
    let cache_minutes = 30;
    let cache_seconds = Duration::from_secs(cache_minutes * 60);
    if let Ok(Some(cache_result)) = fetch_from_cache(&cache_path, cache_seconds) {
        return Ok(cache_result)
    }

    let response = fetch_from_web(url).await?;
    cache_response(&cache_path, &response)?;
    Ok(response)
}

fn fetch_from_cache(
    cache_path: &Path,
    cache_timeout: Duration
) -> anyhow::Result<Option<String>> {
    let cache_file = File::open(cache_path)
        .context(format!("failed to open file: {:?}", cache_path))?;
    let modified = cache_file.metadata()?.modified()?;

    let now = SystemTime::now();
    let cache_age = now.duration_since(modified)?;

    if cache_age < cache_timeout {
        let mut buf_reader = BufReader::new(cache_file);
        let mut cache = String::new();
        buf_reader.read_to_string(&mut cache)?;

        Ok(Some(cache))
    } else {
        Ok(None)
    }
}


fn calculate_cache_path(url: &Url) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    let hash = hasher.finish();

    let cache_file_name = format!("mk-rss-{}", hash);
    Path::new("/tmp/").join(cache_file_name)
}

async fn fetch_from_web(url: Url) -> anyhow::Result<String> {
    // Some sites die if we don't provide a user agent, let's just give them the chrome one.
    let chrome_user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36";

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header(reqwest::header::USER_AGENT, chrome_user_agent)
        .send()
        .await?;

    let body = response.text().await?;

    Ok(body)
}

fn cache_response(cache_path: &Path, body: &String) -> anyhow::Result<()> {
    let mut cache_file = File::create(cache_path)
        .context(format!("failed to open file: {:?}", cache_path))?;
    cache_file.write_all(body.as_bytes())?;
    Ok(())
}
