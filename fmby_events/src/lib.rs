use fmby_core::{
    error::Error, rss::RssScheduler, start_background_task, structs::Data,
    tasks::stale_remover::StaleRemover,
};
use poise::serenity_prelude::{self as serenity, FullEvent};
mod bookmark;
mod channels;

pub struct Handler;

#[serenity::async_trait]
impl serenity::EventHandler for Handler {
    async fn dispatch(&self, ctx: &serenity::Context, event: &FullEvent) {
        if let Err(e) = event_handler(ctx, event).await {
            fmby_core::error::event_handler(ctx, e).await;
        }
    }
}

pub async fn event_handler(ctx: &serenity::Context, event: &FullEvent) -> Result<(), Error> {
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
            bookmark::on_reaction_add(ctx, add_reaction).await;
        }
        FullEvent::ThreadCreate {
            thread,
            newly_created,
            ..
        } => {
            channels::link_testing::on_thread_create(ctx, thread, newly_created).await;
        }
        FullEvent::ThreadUpdate { old, new, .. } => {
            channels::link_testing::on_thread_update(ctx, old, new).await;
        }
        _ => {}
    }

    Ok(())
}
