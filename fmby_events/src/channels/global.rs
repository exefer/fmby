use fmby_core::constants::{AUTO_THREAD_MAPPINGS, FmhyChannel};
use fmby_core::{
    structs::Data,
    utils::{db::WikiUrlFinder, formatters::UrlFormatter, url::extract_urls},
};
use fmby_entities::{prelude::*, sea_orm_active_enums::WikiUrlStatus, wiki_urls};
use poise::serenity_prelude::{
    Color, CreateAllowedMentions, CreateEmbed, CreateMessage, CreateThread, Message,
    MessageReference, prelude::*,
};
use sea_orm::{ActiveValue::*, IntoActiveModel, Iterable, prelude::*, sea_query::OnConflict};

fn is_add_links_channel(id: u64) -> bool {
    id == FmhyChannel::ADD_LINKS || id == FmhyChannel::NSFW_ADD_LINKS
}

fn is_remove_sites_channel(id: u64) -> bool {
    id == FmhyChannel::REMOVE_SITES
        || id == FmhyChannel::NSFW_REMOVED
        || id == FmhyChannel::DEAD_SITES
}

fn is_recently_added_channel(id: u64) -> bool {
    id == FmhyChannel::RECENTLY_ADDED || id == FmhyChannel::NSFW_RECENTLY_ADDED
}

pub async fn on_message(ctx: &Context, message: &Message) {
    for (channel_id, needle) in AUTO_THREAD_MAPPINGS.iter() {
        if message.channel_id.get() == *channel_id
            && needle.is_none_or(|n| message.content.contains(n))
        {
            let _ = message
                .channel_id
                .expect_channel()
                .create_thread_from_message(
                    &ctx.http,
                    message.id,
                    CreateThread::new("Please keep discussions in here!")
                        .audit_log_reason("Auto thread created by FMBY bot"),
                )
                .await;
            return;
        }
    }

    if message.author.bot() {
        return;
    }

    let Some(urls) = extract_urls(&message.content, true) else {
        return;
    };

    match urls
        .find_wiki_url_entries(&ctx.data::<Data>().database.pool)
        .await
    {
        Ok(wiki_entries) if !wiki_entries.is_empty() => match message.channel_id.get() {
            id if is_remove_sites_channel(id) || is_recently_added_channel(id) => {
                let status = if is_remove_sites_channel(id) {
                    WikiUrlStatus::Removed
                } else {
                    WikiUrlStatus::Added
                };

                for mut entry in wiki_entries
                    .into_iter()
                    .map(IntoActiveModel::into_active_model)
                {
                    entry.user_id = Set(Some(message.author.id.get() as i64));
                    entry.message_id = Set(Some(message.id.get() as i64));
                    entry.channel_id = Set(Some(id as i64));
                    entry.status = Set(status);

                    let _ = entry.update(&ctx.data::<Data>().database.pool).await;
                }
            }
            _ => {
                let mut embed = CreateEmbed::new().title("Warning").color(Color::ORANGE);

                for status in WikiUrlStatus::iter() {
                    if let Some(formatted) = wiki_entries.format_for_embed(&status) {
                        let title = match status {
                            WikiUrlStatus::Added => "Link(s) already in the wiki:",
                            WikiUrlStatus::Pending => "Links(s) already in queue:",
                            WikiUrlStatus::Removed => "Links(s) previously removed from the wiki:",
                        };

                        embed = embed.field(title, formatted, false);
                    }
                }

                let _ = message
                    .channel_id
                    .send_message(
                        &ctx.http,
                        CreateMessage::new()
                            .add_embed(embed)
                            .reference_message(MessageReference::from(message))
                            .allowed_mentions(CreateAllowedMentions::new().replied_user(true)),
                    )
                    .await;
            }
        },
        Ok(_) => {
            if !is_add_links_channel(message.channel_id.get()) {
                return;
            }

            let _ = WikiUrls::insert_many(
                urls.iter()
                    .map(|url| wiki_urls::ActiveModel {
                        url: Set(url.to_owned()),
                        user_id: Set(Some(message.author.id.get() as i64)),
                        guild_id: Set(message.guild_id.map(|g| g.get() as i64)),
                        channel_id: Set(Some(message.channel_id.get() as i64)),
                        message_id: Set(Some(message.id.get() as i64)),
                        status: Set(WikiUrlStatus::Pending),
                        ..Default::default()
                    })
                    .collect::<Vec<_>>(),
            )
            .on_conflict(
                OnConflict::column(wiki_urls::Column::Url)
                    .do_nothing()
                    .to_owned(),
            )
            .do_nothing()
            .exec(&ctx.data::<Data>().database.pool)
            .await;
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                guild_id = ?message.guild_id.map(|g| g.get()),
                channel_id = %message.channel_id.get(),
                message_id = %message.id.get(),
                user_id = %message.author.id.get(),
                urls = ?urls,
                "Failed to insert wiki URLs into the database"
            );
        }
    }
}
