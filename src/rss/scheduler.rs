use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use futures::stream::FuturesUnordered;
use poise::serenity_prelude::{
    Context, CreateEmbed, CreateEmbedFooter, CreateMessage, GenericChannelId, Timestamp,
    async_trait, futures,
};
use sea_orm::TryIntoModel;

use crate::background_task::BackgroundTask;
use crate::entities::{rss_feed_entries, rss_feeds};
use crate::error::Error;
use crate::rss::{RssFetcher, RssManager};
use crate::types::Data;

pub struct RssScheduler {
    ctx: Context,
    rss_manager: RssManager,
}

impl RssScheduler {
    pub fn new(ctx: Context) -> Self {
        let rss_manager = RssManager::new(ctx.data::<Data>().pool.clone());
        Self { ctx, rss_manager }
    }

    async fn check_all_feeds(&self) -> Result<(), Error> {
        let feeds = self.rss_manager.get_feeds_to_check().await?;

        if feeds.is_empty() {
            return Ok(());
        }

        let semaphore = Arc::new(tokio::sync::Semaphore::new(
            self.ctx
                .data_ref::<Data>()
                .rss_config
                .settings
                .max_concurrent_checks,
        ));

        let mut tasks = FuturesUnordered::new();

        for feed in feeds {
            let sem = Arc::clone(&semaphore);

            let task = async move {
                let _permit = sem.acquire().await.unwrap();
                self.check_single_feed(feed).await
            };

            tasks.push(task);
        }

        while tasks.next().await.is_some() {}

        Ok(())
    }

    async fn check_single_feed(&self, feed: rss_feeds::Model) -> Result<(), Error> {
        let _ = self.rss_manager.update_last_checked_at(feed.id).await;

        let data = self.ctx.data_ref::<Data>();
        let fetcher = RssFetcher::new(&data.rss_config);

        let Ok(entries) = fetcher.fetch_feed(&feed).await else {
            return Ok(());
        };

        if entries.is_empty() {
            return Ok(());
        }

        let max_entries = data.rss_config.settings.max_entries_per_check;

        let entries: Vec<_> = if self
            .rss_manager
            .get_feed_entry_count(feed.id)
            .await
            .unwrap_or(0)
            == 0
        {
            entries.into_iter().take(max_entries).collect()
        } else {
            let cutoff = self
                .rss_manager
                .get_oldest_entry_published_at(feed.id)
                .await
                .unwrap_or(None);

            if let Some(cutoff) = cutoff {
                entries
                    .into_iter()
                    .filter(|e| match e.published_at.as_ref() {
                        Some(date) => *date > cutoff,
                        None => true,
                    })
                    .collect()
            } else {
                entries
            }
        };

        let entries_to_post: Vec<_> = if data.rss_config.settings.debug_force_post {
            entries
                .into_iter()
                .filter_map(|e| e.try_into_model().ok())
                .collect::<Vec<_>>()
                .into_iter()
                .take(max_entries)
                .rev()
                .collect()
        } else {
            let new_entries = self.rss_manager.insert_feed_entries(entries).await?;
            if new_entries.is_empty() {
                return Ok(());
            }
            new_entries.into_iter().take(max_entries).rev().collect()
        };

        for entry in entries_to_post {
            self.post_entry_to_discord(&feed, entry).await?;
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        Ok(())
    }

    async fn post_entry_to_discord(
        &self,
        feed: &rss_feeds::Model,
        entry: rss_feed_entries::Model,
    ) -> Result<(), Error> {
        let timestamp = entry.published_at.unwrap_or(entry.created_at);
        let timestamp_str = timestamp.to_rfc3339();
        let data = self.ctx.data_ref::<Data>();

        let mut embed =
            CreateEmbed::new()
                .title(&entry.title)
                .color(data.rss_config.embed.color)
                .timestamp(Timestamp::parse(&timestamp_str).unwrap_or_else(|_| {
                    Timestamp::from_millis(timestamp.timestamp_millis()).unwrap()
                }));

        if let Some(link) = &entry.link {
            embed = embed.url(link);
        }

        if let Some(description) = &entry.description {
            embed = if description.len() > data.rss_config.embed.max_description_length {
                embed.description(format!(
                    "{}...",
                    &description[..data.rss_config.embed.max_description_length]
                ))
            } else {
                embed.description(description)
            };
        }

        if let Some(thumbnail_url) = &entry.thumbnail_url {
            embed = embed.image(thumbnail_url, None);
        }

        embed = embed.footer(CreateEmbedFooter::new(format!("📡 {}", feed.name)));

        let message = GenericChannelId::new(feed.channel_id as u64)
            .send_message(&self.ctx.http, CreateMessage::new().add_embed(embed))
            .await?;

        let _ = self
            .rss_manager
            .update_entry_message_id(entry.id, message.id.get())
            .await;

        Ok(())
    }
}

#[async_trait]
impl BackgroundTask for RssScheduler {
    async fn init(ctx: Context) -> Result<Self, Error> {
        Ok(Self::new(ctx))
    }

    fn interval(&mut self) -> Duration {
        Duration::from_secs(60)
    }

    async fn run(&mut self) {
        let _ = self.check_all_feeds().await;
    }

    fn timeout(&mut self) -> Option<Duration> {
        None
    }
}
