use crate::rss::RssConfig;
use anyhow::anyhow;
use fmby_entities::{rss_feed_entries, rss_feeds};
use regex::Regex;
use sea_orm::{ActiveValue::*, prelude::*, sqlx::types::chrono::Utc};
use std::{sync::LazyLock, time::Duration};

pub struct RssFetcher {
    client: reqwest::Client,
}

impl RssFetcher {
    pub fn new(config: &RssConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.fetcher.http_timeout_seconds))
            .user_agent(env!("CARGO_PKG_NAME"))
            .build()
            .expect("HTTP client creation failed");

        Self { client }
    }

    pub async fn fetch_feed(
        &self,
        feed: &rss_feeds::Model,
    ) -> anyhow::Result<Vec<rss_feed_entries::ActiveModel>> {
        tracing::debug!("Starting feed retrieval for: {} at {}", feed.name, feed.url);

        let response = self.client.get(&feed.url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Request failed with status {}: {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            ));
        }

        let content = response.text().await?;
        let parsed_feed = feed_rs::parser::parse(content.as_bytes())
            .map_err(|e| anyhow!("Feed parsing error: {}", e))?;

        tracing::info!(
            "Successfully parsed '{}' containing {} entries",
            parsed_feed
                .title
                .as_ref()
                .map_or("Unnamed feed", |t| t.content.as_str()),
            parsed_feed.entries.len()
        );

        let mut entries: Vec<_> = parsed_feed
            .entries
            .into_iter()
            .map(|entry| self.convert_to_active_model(feed.id, entry))
            .collect();

        entries.sort_by(
            |a, b| match (a.published_at.as_ref(), b.published_at.as_ref()) {
                (Some(a_date), Some(b_date)) => b_date.cmp(a_date),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            },
        );

        Ok(entries)
    }

    fn convert_to_active_model(
        &self,
        feed_id: Uuid,
        entry: feed_rs::model::Entry,
    ) -> rss_feed_entries::ActiveModel {
        let entry_id = if entry.id.is_empty() {
            entry.links.first().map_or_else(
                || {
                    let timestamp = entry
                        .published
                        .or(entry.updated)
                        .unwrap_or_else(Utc::now)
                        .to_rfc3339();
                    let title = entry
                        .title
                        .as_ref()
                        .map_or("untitled", |t| t.content.as_str());
                    format!("{}_{}", title, timestamp)
                },
                |link| link.href.clone(),
            )
        } else {
            entry.id
        };

        let title = entry
            .title
            .map_or_else(|| "Untitled".to_owned(), |t| t.content);

        let link = entry.links.first().map(|l| l.href.clone());

        let description = entry
            .summary
            .as_ref()
            .map(|s| clean_html(&s.content))
            .or_else(|| {
                entry
                    .content
                    .as_ref()
                    .and_then(|c| c.body.as_ref())
                    .map(|body| clean_html(body))
            })
            .filter(|s| !s.trim().is_empty());

        let thumbnail_url = entry
            .media
            .first()
            .and_then(|m| {
                m.thumbnails
                    .first()
                    .map(|m| m.image.uri.clone())
                    .or_else(|| {
                        let m = m.content.first()?;
                        if let (Some(url), Some(content_type)) = (&m.url, &m.content_type)
                            && content_type.ty() == "image"
                        {
                            Some(url.to_string())
                        } else {
                            None
                        }
                    })
            })
            .or_else(|| {
                entry
                    .summary
                    .as_ref()
                    .and_then(|s| find_first_image(&s.content))
                    .filter(|url| !url.is_empty())
            })
            .or_else(|| find_first_image(entry.content.as_ref().and_then(|c| c.body.as_ref())?));

        rss_feed_entries::ActiveModel {
            id: Set(Uuid::new_v4()),
            feed_id: Set(feed_id),
            entry_id: Set(entry_id),
            title: Set(title),
            link: Set(link),
            description: Set(description),
            thumbnail_url: Set(thumbnail_url),
            published_at: Set(entry.published.or(entry.updated).map(Into::into)),
            ..Default::default()
        }
    }

    pub async fn validate_feed_url(&self, url: &str) -> anyhow::Result<String> {
        tracing::debug!("Validating feed URL: {}", url);

        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Validation failed - HTTP {}: {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            ));
        }

        let content = response.text().await?;
        let parsed_feed = feed_rs::parser::parse(content.as_bytes())
            .map_err(|e| anyhow!("Feed format validation failed: {}", e))?;

        Ok(parsed_feed
            .title
            .map(|t| t.content)
            .filter(|t| !t.trim().is_empty())
            .unwrap_or_else(|| "RSS Feed".to_owned()))
    }
}

static NUMERIC_ENTITY_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"&#(\d+);").unwrap());

/// Converts HTML character entities to their corresponding Unicode characters
/// Processes both numeric references (&#39;) and named entities (&amp;)
fn decode_html_entities(html: &str) -> String {
    // Handle numeric character codes first
    let result = NUMERIC_ENTITY_RE.replace_all(html, |caps: &regex::Captures| {
        caps.get(1)
            .and_then(|m| m.as_str().parse::<u32>().ok())
            .and_then(char::from_u32)
            .map_or_else(
                || caps.get(0).unwrap().as_str().to_owned(),
                |ch| ch.to_string(),
            )
    });

    // Replace common named HTML entities
    result
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
}

/// Removes HTML markup and decodes entities from text content
/// Returns clean text suitable for display or storage
fn clean_html(html: &str) -> String {
    let decoded = decode_html_entities(html);
    let mut result = String::new();
    let mut in_tag = false;

    // Strip out HTML tags while preserving text content
    for ch in decoded.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }

    // Normalize whitespace in the final result
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

static IMG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<img[^>]+src\s*=\s*["']([^"']+)["'][^>]*>"#).unwrap());

/// Locates the first image source URL within HTML markup
/// Searches for img tag src attributes and validates URL format
fn find_first_image(html: &str) -> Option<String> {
    let decoded = decode_html_entities(html);

    // Use regex to find img elements with src attributes
    // Validate and return first valid image URL found
    IMG_RE
        .captures(&decoded)?
        .get(1)
        .map(|m| m.as_str().to_owned())
        .filter(|url| {
            url.starts_with("http://") || url.starts_with("https://") || url.starts_with("//")
        })
}
