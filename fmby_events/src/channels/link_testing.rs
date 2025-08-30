use fmby_core::{
    constants::{FmhyChannel, link_testing::ForumTag},
    utils::url::extract_urls,
};
use poise::serenity_prelude::{small_fixed_array::FixedArray, *};
use std::collections::HashSet;

fn is_thread_in_link_testing(thread: &GuildThread) -> bool {
    thread.parent_id.get() == FmhyChannel::LinkTesting.id()
}

pub async fn on_thread_create(ctx: &Context, thread: &GuildThread, newly_created: &Option<bool>) {
    if !is_thread_in_link_testing(thread) {
        return;
    };

    if let Some(message_id) = thread.base.last_message_id
        && let Ok(message) = thread.id.widen().message(&ctx.http, message_id).await
        && let Some(_urls) = extract_urls(&message.content)
    {
        // TODO: Maybe update the database records with the first message ID and the thread ID
    }

    if *newly_created == Some(true) {
        let builder = CreateMessage::new().content(format!(
            "Thread opened by {} - join in, share your thoughts, and keep the discussion going!",
            thread.owner_id.mention()
        ));

        let _ = thread.send_message(&ctx.http, builder).await;
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
            let content = match tag.get() {
                x if x == ForumTag::Rejected.id() && closing => {
                    Some(format!("{}: thread closed as rejected", owner))
                }
                x if x == ForumTag::Added.id() && closing => Some(format!(
                    "{}: thread closed as approved; link(s) will be added to the wiki.",
                    owner
                )),
                x if x == ForumTag::Rejected.id() && !closing => Some(format!(
                    "{}: your previously rejected thread has been reopened; feel free to continue discussing and defending the link(s) you were testing.",
                    owner
                )),
                _ => None,
            };
            if let Some(content) = content {
                let _ = new
                    .send_message(&ctx.http, CreateMessage::new().content(content))
                    .await;
            }
        }
    }
}
