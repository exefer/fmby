use crate::{
    BackgroundTask,
    error::Error,
    rss::{RssFetcher, RssManager},
    structs::Data,
};
use fmby_entities::{rss_feed_entries, rss_feeds};
use poise::serenity_prelude::{
    self as serenity, CreateEmbed, CreateEmbedFooter, CreateMessage, GenericChannelId, Timestamp,
    prelude::*,
};
use sea_orm::TryIntoModel;
use std::{sync::Arc, time::Duration};

pub struct RssScheduler {
    ctx: Context,
    rss_manager: RssManager,
}
impl RssScheduler {
    pub fn new(ctx: Context) -> Self {
        let rss_manager = RssManager::new(ctx.data::<Data>().database.pool.clone().into());
        Self { ctx, rss_manager }
    }

    async fn check_all_feeds(&self) -> Result<(), Error> {
        let feeds = self.rss_manager.get_feeds_to_check().await?;
        if feeds.is_empty() {
            return Ok(());
        }

        tracing::info!("Processing {} RSS feed(s) for new content", feeds.len());

        let semaphore = Arc::new(tokio::sync::Semaphore::new(
            self.ctx
                .data::<Data>()
                .rss_config
                .settings
                .max_concurrent_checks,
        ));

        let handles: Vec<_> = feeds
            .into_iter()
            .map(|feed| {
                let ctx_clone = self.ctx.clone();
                let rss_manager_clone =
                    RssManager::new(self.ctx.data::<Data>().database.pool.clone().into());
                let semaphore_clone = semaphore.clone();

                tokio::spawn(async move {
                    let _permit = semaphore_clone.acquire().await.unwrap();
                    if let Err(e) =
                        Self::check_single_feed(ctx_clone, rss_manager_clone, feed).await
                    {
                        tracing::error!("RSS feed check failed with error: {}", e);
                    }
                })
            })
            .collect();

        for handle in handles {
            let _ = handle.await;
        }
        Ok(())
    }

    async fn check_single_feed(
        ctx: Context,
        rss_manager: RssManager,
        feed: rss_feeds::Model,
    ) -> Result<(), Error> {
        tracing::info!("Fetching RSS feed: '{}' at {}", feed.name, feed.url);
        let _ = rss_manager.update_last_checked_at(feed.id).await;

        let fetcher = RssFetcher::new(&ctx.data::<Data>().rss_config);
        let entries = match fetcher.fetch_feed(&feed).await {
            Ok(entries) => entries,
            Err(e) => {
                tracing::warn!("Unable to retrieve RSS feed '{}': {}", feed.name, e);
                return Ok(());
            }
        };

        if entries.is_empty() {
            tracing::info!("RSS feed '{}' contains no entries", feed.name);
            return Ok(());
        }

        let data = ctx.data_ref::<Data>();
        let max_entries = data.rss_config.settings.max_entries_per_check;

        let entries_to_post: Vec<_> = if data.rss_config.settings.debug_force_post {
            tracing::info!(
                "DEBUG MODE: Force-posting {} entries from '{}' (may include previously processed items)",
                entries.len().min(max_entries),
                feed.name
            );
            entries
                .into_iter()
                .filter_map(|e| e.try_into_model().ok())
                .collect::<Vec<_>>()
                .into_iter()
                .take(max_entries)
                .rev()
                .collect()
        } else {
            let new_entries = rss_manager.insert_feed_entries(entries).await?;
            if new_entries.is_empty() {
                tracing::info!(
                    "All entries from '{}' have been previously processed",
                    feed.name
                );
                return Ok(());
            }
            tracing::info!(
                "Discovered {} fresh entries in RSS feed '{}'",
                new_entries.len(),
                feed.name
            );
            new_entries.into_iter().take(max_entries).rev().collect()
        };

        for entry in entries_to_post {
            Self::post_entry_to_discord(ctx.clone(), &rss_manager, &feed, entry).await?;

            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        Ok(())
    }

    async fn post_entry_to_discord(
        ctx: Context,
        rss_manager: &RssManager,
        feed: &rss_feeds::Model,
        entry: rss_feed_entries::Model,
    ) -> Result<(), Error> {
        let timestamp = entry.published_at.unwrap_or(entry.created_at);
        let timestamp_str = timestamp.to_rfc3339();
        let data = ctx.data_ref::<Data>();

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

        if let Some(image_url) = &entry.image_url {
            embed = embed.image(image_url);
        }

        embed = embed.footer(CreateEmbedFooter::new(format!("📡 {}", feed.name)));

        let message = GenericChannelId::new(feed.channel_id as u64)
            .send_message(&ctx.http, CreateMessage::new().add_embed(embed))
            .await?;

        if let Err(e) = rss_manager
            .update_entry_message_id(entry.id, message.id.get())
            .await
        {
            tracing::warn!("Could not store message ID for RSS feed entry: {}", e);
        }

        tracing::info!(
            "Successfully delivered RSS feed entry '{}' to channel {}",
            entry.title,
            feed.channel_id
        );
        Ok(())
    }
}

#[serenity::async_trait]
impl BackgroundTask for RssScheduler {
    async fn init(ctx: Context) -> Result<Self, Error> {
        Ok(Self::new(ctx))
    }

    fn interval(&mut self) -> Duration {
        Duration::from_secs(60)
    }

    async fn run(&mut self) {
        if let Err(e) = self.check_all_feeds().await {
            tracing::error!("Error in RSS scheduler: {}", e);
        }
    }

    fn timeout(&mut self) -> Option<Duration> {
        None
    }
}
