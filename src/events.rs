use poise::serenity_prelude::{Context, EventHandler, FullEvent, async_trait};

use crate::background_task::start_background_task;
use crate::channels;
use crate::error::Error;
use crate::rss::RssScheduler;
use crate::stale_remover::StaleRemover;
use crate::types::Data;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn dispatch(&self, ctx: &Context, event: &FullEvent) {
        if let Err(e) = event_handler(ctx, event).await {
            crate::error::event_handler(ctx, e).await;
        }
    }
}

pub async fn event_handler(ctx: &Context, event: &FullEvent) -> Result<(), Error> {
    match event {
        FullEvent::Ready { data_about_bot, .. } => {
            let data = ctx.data_ref::<Data>();
            let shard_count = ctx.cache.shard_count();
            let is_last_shard = (ctx.shard_id.0 + 1) == shard_count.get();

            if is_last_shard
                && !data
                    .has_started
                    .swap(true, std::sync::atomic::Ordering::SeqCst)
            {
                println!("Logged in as {}", data_about_bot.user.tag());

                start_background_task::<RssScheduler>(ctx).await;
                start_background_task::<StaleRemover>(ctx).await;
            }
        }
        FullEvent::Message { new_message, .. } => {
            channels::global::on_message(ctx, new_message).await;
        }
        FullEvent::ReactionAdd { add_reaction, .. } => {
            channels::global::on_reaction_add(ctx, add_reaction).await;
        }
        FullEvent::ThreadUpdate { old, new, .. } => {
            channels::link_testing::on_thread_update(ctx, old.as_ref(), new).await;
        }
        _ => {}
    }

    Ok(())
}
