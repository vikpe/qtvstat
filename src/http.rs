use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow as e;
use dashmap::DashMap;

/// Get demo filenames from a single server
pub async fn demo_filenames(address: &str, timeout: Duration) -> anyhow::Result<Vec<String>> {
    let now = std::time::Instant::now();
    let client = reqwest::Client::new();
    let url = demo_filenames_url(address);

    match client.get(&url).timeout(timeout).send().await {
        Ok(res) if !res.status().is_success() => {
            Err(e!("qtvstat::demo_filenames: unable to fetch {}", url))
        }
        Err(e) => Err(e!(
            "qtvstat::demo_filenames: unable to fetch ({}/{:?} timeout) {} - {}",
            now.elapsed().as_millis(),
            timeout.as_millis(),
            url,
            e
        )),
        Ok(res) => {
            let filenames = res
                .text()
                .await?
                .split_whitespace()
                .map(|line| line.to_string())
                .filter(|line| line.ends_with(".mvd"))
                .collect::<Vec<String>>();

            Ok(filenames)
        }
    }
}

/// Get demo URLs from a single server
pub async fn demo_urls(address: &str, timeout: Duration) -> anyhow::Result<Vec<String>> {
    let urls: Vec<String> = demo_filenames(address, timeout)
        .await?
        .iter()
        .map(|filename| filename_to_url(address, filename))
        .collect();

    Ok(urls)
}

/// Get demo filenames from a multiple servers (async, in parallel)
pub async fn demo_filenames_per_address(
    addresses: &[String],
    timeout: Duration,
) -> HashMap<String, anyhow::Result<Vec<String>>> {
    let mut task_handles = vec![];
    let result_arc: Arc<DashMap<String, anyhow::Result<Vec<String>>>> = Default::default();

    for address in addresses {
        let result_arc = Arc::clone(&result_arc);
        let address = address.clone();

        let task = tokio::spawn(async move {
            result_arc.insert(
                address.clone(),
                demo_filenames(address.as_str(), timeout).await,
            );
        });
        task_handles.push(task);
    }
    futures::future::join_all(task_handles).await;

    // convert DashMap to HashMap
    let mut result: HashMap<String, anyhow::Result<Vec<String>>> = HashMap::new();

    for (k, v) in Arc::into_inner(result_arc).unwrap().into_iter() {
        result.insert(k, v);
    }

    result
}

/// Get the URL to download a demo
pub fn filename_to_url(address: &str, filename: &str) -> String {
    format!("http://{}/dl/demos/{}", address, filename)
}

fn demo_filenames_url(address: &str) -> String {
    format!("http://{}/demo_filenames", address)
}


#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_filename_to_url() {
        assert_eq!(
            filename_to_url("example.com", "foo.mvd"),
            "http://example.com/dl/demos/foo.mvd"
        );
    }

    #[test]
    fn test_demo_filenames_url() {
        assert_eq!(demo_filenames_url("example.com"), "http://example.com/demo_filenames");
    }
}