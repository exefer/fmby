use crate::{
    BackgroundTask,
    constants::{FmhyChannel, link_testing::ForumTag},
    error::Error,
    structs::Data,
};
use fmby_entities::{prelude::*, sea_orm_active_enums::WikiUrlStatus, wiki_urls};
use poise::serenity_prelude::{self as serenity, Channel, Context, GenericChannelId, MessageId};
use sea_orm::{QueryOrder, prelude::*};
use std::time::Duration;

pub struct StaleRemover {
    ctx: Context,
}

impl StaleRemover {
    pub fn new(ctx: Context) -> Self {
        Self { ctx }
    }
}

#[serenity::async_trait]
impl BackgroundTask for StaleRemover {
    async fn init(ctx: Context) -> Result<Self, Error> {
        Ok(Self::new(ctx))
    }

    fn interval(&mut self) -> Duration {
        Duration::from_secs(60 * 60 * 24) // 1 day
    }

    async fn run(&mut self) {
        if let Ok(entries) = WikiUrls::find()
            .filter(wiki_urls::Column::Status.eq(WikiUrlStatus::Pending))
            .filter(
                Expr::col(wiki_urls::Column::CreatedAt)
                    .lt(Expr::current_timestamp().sub(Expr::cust("INTERVAL '14 days'"))),
            )
            .order_by_asc(wiki_urls::Column::CreatedAt)
            .all(&self.ctx.data_ref::<Data>().database.pool)
            .await
        {
            for entry in entries {
                let (Some(cid), Some(mid)) = (entry.channel_id, entry.message_id) else {
                    continue;
                };
                let (cid, mid) = (cid as u64, mid as u64);

                if cid == FmhyChannel::ADD_LINKS
                    && self
                        .ctx
                        .http
                        .get_message(GenericChannelId::new(cid), MessageId::new(mid))
                        .await
                        .is_ok()
                {
                    continue;
                }

                if cid != FmhyChannel::ADD_LINKS
                    && let Ok(Channel::GuildThread(thread)) =
                        self.ctx.http.get_channel(GenericChannelId::new(cid)).await
                {
                    if thread.parent_id.get() != FmhyChannel::LINK_TESTING {
                        continue;
                    }

                    if !thread
                        .applied_tags
                        .iter()
                        .any(|t| matches!(t.get(), ForumTag::ADDED | ForumTag::REJECTED))
                    {
                        continue;
                    }
                }

                let _ = entry
                    .delete(&self.ctx.data_ref::<Data>().database.pool)
                    .await;
            }
        }
    }

    fn timeout(&mut self) -> Option<Duration> {
        None
    }
}
