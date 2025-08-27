use fmby_core::constants::AUTO_THREAD_MAPPINGS;
use poise::serenity_prelude::*;

pub async fn on_message(ctx: &Context, message: &Message) {
    for (channel_id, needle) in AUTO_THREAD_MAPPINGS.iter() {
        if message.channel_id != *channel_id {
            continue;
        }

        if needle.is_none_or(|n| message.content.contains(n)) {
            let channel_id = message.channel_id.expect_channel();
            let _ = channel_id
                .create_thread_from_message(
                    &ctx.http,
                    message.id,
                    CreateThread::new("Please keep discussions in here!")
                        .audit_log_reason("Auto thread created by FMBY bot"),
                )
                .await;
        }
    }
}
