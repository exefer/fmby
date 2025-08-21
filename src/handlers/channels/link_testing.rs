use crate::{
    config::LinkTestingMessages,
    constants::{DevChannel, link_testing::ForumTag},
    utils::url::extract_urls,
};
use entity::prelude::*;
use poise::serenity_prelude::{self as serenity, Mentionable};
use std::collections::HashSet;

pub struct LinkTestingHandler;

impl LinkTestingHandler {
    fn is_thread_in_link_testing(thread: &serenity::GuildChannel) -> bool {
        thread.parent_id.map(|c| c.get()).unwrap_or_default() == DevChannel::LinkTesting.id()
    }
}

#[serenity::async_trait]
impl serenity::EventHandler for LinkTestingHandler {
    async fn thread_create(&self, ctx: serenity::Context, thread: serenity::GuildChannel) {
        let is_new_testing_thread =
            Self::is_thread_in_link_testing(&thread) && thread.member_count == Some(1);

        if !is_new_testing_thread {
            return;
        };

        let message = thread
            .messages(&ctx.http, serenity::GetMessages::new().limit(1))
            .await
            .map(|msgs| msgs.first().cloned())
            .unwrap_or(None);

        if let Some(message) = message
            && let Some(urls) = extract_urls(&message.content)
        {
            // TODO: Maybe update the database records with the first message ID and the thread ID
        }

        if let Err(_err) = thread
            .say(
                &ctx.http,
                LinkTestingMessages::get_thread_create_welcome(
                    &ctx.data,
                    thread.owner_id.unwrap().mention(),
                )
                .await,
            )
            .await
        {
            todo!()
        }
    }

    async fn thread_update(
        &self,
        ctx: serenity::Context,
        old: Option<serenity::GuildChannel>,
        new: serenity::GuildChannel,
    ) {
        if !Self::is_thread_in_link_testing(&new) {
            return;
        }

        let old_tags: HashSet<_> = old
            .map(|c| c.applied_tags)
            .unwrap_or_default()
            .into_iter()
            .collect();
        let new_tags: HashSet<_> = new.applied_tags.clone().into_iter().collect();
        let owner = new.owner_id.unwrap().mention();

        for (tags, closing) in [
            (new_tags.difference(&old_tags), true),
            (old_tags.difference(&new_tags), false),
        ] {
            for tag in tags {
                let msg = match tag.get() {
                    x if x == ForumTag::Rejected.id() && closing => Some(
                        LinkTestingMessages::get_thread_update_rejected(&ctx.data, owner).await,
                    ),
                    x if x == ForumTag::Added.id() && closing => Some(
                        LinkTestingMessages::get_thread_update_approved(&ctx.data, owner).await,
                    ),
                    x if x == ForumTag::Rejected.id() && !closing => Some(
                        LinkTestingMessages::get_thread_update_reopened(&ctx.data, owner).await,
                    ),
                    _ => None,
                };
                if let Some(text) = msg
                    && let Err(_) = new.say(&ctx.http, text).await
                {
                    todo!()
                };
            }
        }
    }
}
