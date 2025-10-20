use fmby_core::{
    constants::{FmhyChannel, link_testing::ForumTag},
    structs::Data,
    utils::{
        db::{get_wiki_urls_by_urls, update_wiki_urls_with_message},
        url::extract_urls,
    },
};
use fmby_entities::sea_orm_active_enums::WikiUrlStatus;
use poise::serenity_prelude::{CreateMessage, GetMessages, GuildThread, prelude::*};
use std::collections::HashSet;

pub async fn on_thread_create(ctx: &Context, thread: &GuildThread, _newly_created: Option<&bool>) {
    if thread.parent_id.get() != FmhyChannel::LINK_TESTING {
        return;
    }

    if let Some(message) = thread
        .id
        .widen()
        .messages(&ctx.http, GetMessages::new().limit(1))
        .await
        .ok()
        .and_then(|m| m.into_iter().next())
        && let Some(urls) = extract_urls(&message.content)
        && let Some(entries) = get_wiki_urls_by_urls(&urls, &ctx.data::<Data>().database.pool).await
    {
        update_wiki_urls_with_message(
            entries,
            &message,
            WikiUrlStatus::Pending,
            &ctx.data::<Data>().database.pool,
        )
        .await;
    }
}

pub async fn on_thread_update(ctx: &Context, old: Option<&GuildThread>, new: &GuildThread) {
    if new.parent_id.get() != FmhyChannel::LINK_TESTING {
        return;
    }

    let Some(old) = old else {
        return;
    };

    let old_tags: HashSet<_> = old.applied_tags.iter().copied().collect();
    let new_tags: HashSet<_> = new.applied_tags.iter().copied().collect();

    if old_tags == new_tags {
        return;
    }

    let owner = new.owner_id.mention();

    for (tags, closing) in [
        (new_tags.difference(&old_tags), true),
        (old_tags.difference(&new_tags), false),
    ] {
        for tag in tags {
            let content = match (tag.get(), closing) {
                (ForumTag::REJECTED, true) => {
                    Some(format!("{}: thread closed as rejected.", owner))
                }
                (ForumTag::ADDED, true) => Some(format!(
                    "{}: thread closed as approved; links will be added to the wiki.",
                    owner
                )),
                (ForumTag::REJECTED, false) => Some(format!(
                    "{}: your previously rejected thread has been reopened; feel free to continue discussing and defending the links you were testing.",
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
