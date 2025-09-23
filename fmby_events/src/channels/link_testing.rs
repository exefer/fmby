use fmby_core::{
    constants::{FmhyChannel, link_testing::ForumTag},
    structs::Data,
    utils::{db::WikiUrlFinder, url::extract_urls},
};
use fmby_entities::sea_orm_active_enums::WikiUrlStatus;
use poise::serenity_prelude::{
    CreateMessage, GuildThread, prelude::*, small_fixed_array::FixedArray,
};
use sea_orm::{ActiveValue::*, IntoActiveModel, prelude::*};
use std::collections::HashSet;

fn is_thread_in_link_testing(thread: &GuildThread) -> bool {
    thread.parent_id.get() == FmhyChannel::LINK_TESTING
}

pub async fn on_thread_create(ctx: &Context, thread: &GuildThread, newly_created: &Option<bool>) {
    if !is_thread_in_link_testing(thread) {
        return;
    };

    if let Some(message_id) = thread.base.last_message_id
        && let Ok(message) = thread.id.widen().message(&ctx.http, message_id).await
        && let Some(urls) = extract_urls(&message.content, true)
        && let Ok(entries) = urls
            .find_wiki_url_entries(&ctx.data::<Data>().database.pool)
            .await
    {
        for mut entry in entries.into_iter().map(IntoActiveModel::into_active_model) {
            entry.user_id = Set(Some(message.author.id.get() as i64));
            entry.message_id = Set(Some(message.id.get() as i64));
            entry.channel_id = Set(Some(thread.id.get() as i64));
            entry.status = Set(WikiUrlStatus::Pending);

            let _ = entry.update(&ctx.data::<Data>().database.pool).await;
        }
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
                x if x == ForumTag::REJECTED && closing => {
                    Some(format!("{}: thread closed as rejected", owner))
                }
                x if x == ForumTag::ADDED && closing => Some(format!(
                    "{}: thread closed as approved; link(s) will be added to the wiki.",
                    owner
                )),
                x if x == ForumTag::REJECTED && !closing => Some(format!(
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
