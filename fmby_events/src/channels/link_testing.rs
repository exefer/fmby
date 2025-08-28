use fmby_core::{
    config::LinkTestingMessages,
    constants::{DevChannel, link_testing::ForumTag},
    utils::url::extract_urls,
};
use poise::serenity_prelude::{small_fixed_array::FixedArray, *};
use std::collections::HashSet;

fn is_thread_in_link_testing(thread: &GuildThread) -> bool {
    thread.parent_id.get() == DevChannel::LinkTesting.id()
}

pub async fn on_thread_create(ctx: &Context, thread: &GuildThread, newly_created: &Option<bool>) {
    if !is_thread_in_link_testing(thread) {
        return;
    };

    if let Some(message_id) = thread.base.last_message_id
        && let Ok(message) = thread.id.widen().message(&ctx.http, message_id).await
        && let Some(urls) = extract_urls(&message.content)
    {
        // TODO: Maybe update the database records with the first message ID and the thread ID
    }

    if *newly_created == Some(true) {
        let _ = thread
            .send_message(
                &ctx.http,
                CreateMessage::new().content(LinkTestingMessages::get_thread_create_welcome(
                    &ctx.data(),
                    thread.owner_id.mention(),
                )),
            )
            .await;
    }
}

pub async fn on_thread_update(ctx: &Context, old: &Option<GuildThread>, new: &GuildThread) {
    if !is_thread_in_link_testing(new) {
        return;
    }

    let old_tags: HashSet<_> = old
        .as_ref()
        .map_or(&FixedArray::default(), |c| &c.applied_tags)
        .iter()
        .copied()
        .collect();
    let new_tags: HashSet<_> = new.applied_tags.iter().copied().collect();
    let owner = new.owner_id.mention();

    for (tags, closing) in [
        (new_tags.difference(&old_tags), true),
        (old_tags.difference(&new_tags), false),
    ] {
        for tag in tags {
            let text = match tag.get() {
                x if x == ForumTag::Rejected.id() && closing => Some(
                    LinkTestingMessages::get_thread_update_rejected(&ctx.data(), owner),
                ),
                x if x == ForumTag::Added.id() && closing => Some(
                    LinkTestingMessages::get_thread_update_approved(&ctx.data(), owner),
                ),
                x if x == ForumTag::Rejected.id() && !closing => Some(
                    LinkTestingMessages::get_thread_update_reopened(&ctx.data(), owner),
                ),
                _ => None,
            };
            if let Some(text) = text
                && let Err(_) = new
                    .send_message(&ctx.http, CreateMessage::new().content(text))
                    .await
            {
                todo!()
            };
        }
    }
}
