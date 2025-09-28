use fmby_core::{
    constants::{FmhyChannel, link_testing::ForumTag},
    structs::Data,
    utils::{db::WikiUrlFinder, url::extract_urls},
};
use fmby_entities::sea_orm_active_enums::WikiUrlStatus;
use poise::serenity_prelude::{CreateMessage, GetMessages, GuildThread, prelude::*};
use sea_orm::{ActiveValue::*, IntoActiveModel, prelude::*, sqlx::types::chrono::Utc};
use std::collections::HashSet;

pub async fn on_thread_create(ctx: &Context, thread: &GuildThread, _newly_created: &Option<bool>) {
    if thread.parent_id.get() != FmhyChannel::LINK_TESTING {
        return;
    };

    if let Some(message) = thread
        .id
        .widen()
        .messages(&ctx.http, GetMessages::new().limit(1))
        .await
        .ok()
        .and_then(|m| m.into_iter().next())
        && let Some(urls) = extract_urls(&message.content, true)
        && let Ok(entries) = urls
            .find_wiki_url_entries(&ctx.data::<Data>().database.pool)
            .await
    {
        for mut entry in entries.into_iter().map(IntoActiveModel::into_active_model) {
            entry.user_id = Set(Some(message.author.id.get() as i64));
            entry.message_id = Set(Some(message.id.get() as i64));
            entry.channel_id = Set(Some(thread.id.get() as i64));
            entry.updated_at = Set(Utc::now().into());
            entry.status = Set(WikiUrlStatus::Pending);

            let _ = entry.update(&ctx.data::<Data>().database.pool).await;
        }
    }
}

pub async fn on_thread_update(ctx: &Context, old: &Option<GuildThread>, new: &GuildThread) {
    if new.parent_id.get() != FmhyChannel::LINK_TESTING {
        return;
    }

    let old_tags: HashSet<_> = old
        .as_ref()
        .map(|o| o.applied_tags.iter().copied())
        .into_iter()
        .flatten()
        .collect();
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
                    "{}: thread closed as approved; link(s) will be added to the wiki.",
                    owner
                )),
                (ForumTag::REJECTED, false) => Some(format!(
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
