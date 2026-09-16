use anyhow::Result;
use dom_smoothie::{Readability, TextMode};
use mime::Mime;
use reqwest::header;
use reqwest_middleware::ClientBuilder;
use reqwest_ssrf_guard::Acl;
use std::str::FromStr;
use tokio::runtime::Runtime;

/// Fetch a URL, extract the main article, and convert it to Markdown with YAML front‑matter.
pub fn url_to_md(url: &str) -> Result<String> {
    let rt = Runtime::new()?;
    rt.block_on(async { inner(url).await })
}

async fn inner(url: &str) -> Result<String> {
    // Build SSRF guard: block private and link-local IPs, allow public.
    let acl = Acl::new().deny_local_network();

    // Build HTTP client with timeout, user‑agent, and SSRF protection via middleware.
    let inner_client = acl.configure(reqwest::Client::builder())
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("nxm-webtomd/1.0")
        .build()?;
    let client = ClientBuilder::new(inner_client)
        .with(acl.clone())
        .build();

    // GET request with HTML accept header.
    let resp = client
        .get(url)
        .header(header::ACCEPT, "text/html,application/xhtml+xml")
        .send()
        .await?;

    // Ensure successful status.
    if !resp.status().is_success() {
        anyhow::bail!("HTTP {}", resp.status());
    }

    // Validate content‑type.
    let content_type = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let mime_type = Mime::from_str(content_type)?;
    if mime_type.type_() != mime::TEXT || mime_type.subtype() != mime::HTML {
        anyhow::bail!("unexpected content type: {}", content_type);
    }

    // Read response with size limit (5 MiB).
    const MAX_BYTES: usize = 5 * 1024 * 1024;
    let bytes = resp.bytes().await?;
    if bytes.len() > MAX_BYTES {
        anyhow::bail!("response too large (>{} MiB)", MAX_BYTES / 1024 / 1024);
    }

    let html = String::from_utf8_lossy(&bytes).into_owned();

    // Parse with dom_smoothie, request Markdown output.
    let mut readability = Readability::new(html, None, None)?;
    readability.config.text_mode = TextMode::Markdown;
    let article = readability.parse()?;

    // Extract metadata.
    let title = article.title.as_str().trim();
    let canonical_url = article
        .url
        .as_deref()
        .unwrap_or(url)
        .trim();
    let byline = article.byline.as_deref().unwrap_or("").trim();
    let published_time = article.published_time.as_deref().unwrap_or("").trim();

    // The extracted Markdown body.
    let markdown_body = article.text_content.to_string();

    // Guard against empty or too‑short extraction.
    if markdown_body.chars().count() < 50 {
        anyhow::bail!("extracted content too short (<50 chars)");
    }

    // Build YAML front‑matter.
    let mut fm = String::new();
    fm.push_str("---\n");
    if !title.is_empty() {
        fm.push_str(&format!("title: {}\n", escape_yaml(title)));
    }
    fm.push_str(&format!("canonical_url: {}\n", escape_yaml(canonical_url)));
    if !byline.is_empty() {
        fm.push_str(&format!("byline: {}\n", escape_yaml(byline)));
    }
    if !published_time.is_empty() {
        fm.push_str(&format!("published_time: {}\n", escape_yaml(published_time)));
    }
    fm.push_str("---\n");

    Ok(format!("{}{}", fm, markdown_body))
}

/// Very simple YAML‑string escaping: wrap in double quotes and escape existing quotes.
fn escape_yaml(s: &str) -> String {
    let escaped = s.replace('"', "\\\"");
    format!("\"{}\"", escaped)
}