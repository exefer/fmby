use std::collections::HashSet;

use poise::serenity_prelude::{CreateMessage, GuildThread, prelude::*};

use crate::constants::FmhyChannel;
use crate::constants::link_testing::ForumTag;

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
                (ForumTag::REJECTED, true) => Some(format!("{owner}: thread closed as rejected.")),
                (ForumTag::ADDED, true) => Some(format!(
                    "{owner}: thread closed as approved; links will be added to the wiki."
                )),
                (ForumTag::REJECTED, false) => Some(format!(
                    "{owner}: your previously rejected thread has been reopened; feel free to continue discussing and defending the links you were testing."
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
