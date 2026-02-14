use std::time::Duration;

use fmby_entities::sea_orm_active_enums::WikiUrlStatus;
use fmby_entities::{prelude::*, wiki_urls};
use poise::serenity_prelude::{Channel, Context, GenericChannelId, MessageId, async_trait};
use sea_orm::{QueryOrder, prelude::*};

use crate::background_task::BackgroundTask;
use crate::constants::FmhyChannel;
use crate::constants::link_testing::ForumTag;
use crate::error::Error;
use crate::structs::Data;

pub struct StaleRemover {
    ctx: Context,
}

impl StaleRemover {
    pub fn new(ctx: Context) -> Self {
        Self { ctx }
    }
}

#[async_trait]
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
                    .lt(Expr::current_timestamp().sub(Expr::cust("INTERVAL '1 day'"))),
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
