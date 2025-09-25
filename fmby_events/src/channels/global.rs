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
use sea_orm::sqlx::types::chrono::Utc;
use sea_orm::{ActiveValue::*, IntoActiveModel, Iterable, prelude::*};

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

    let status = match message.channel_id.get() {
        FmhyChannel::ADD_LINKS | FmhyChannel::NSFW_ADD_LINKS => Some(WikiUrlStatus::Pending),
        FmhyChannel::RECENTLY_ADDED | FmhyChannel::NSFW_RECENTLY_ADDED => {
            Some(WikiUrlStatus::Added)
        }
        FmhyChannel::DEAD_SITES | FmhyChannel::REMOVE_SITES | FmhyChannel::NSFW_REMOVED => {
            Some(WikiUrlStatus::Removed)
        }
        _ => None,
    };

    let entries = urls
        .find_wiki_url_entries(&ctx.data::<Data>().database.pool)
        .await;

    if let Ok(entries) = entries {
        if !entries.is_empty() {
            match status {
                Some(WikiUrlStatus::Added) | Some(WikiUrlStatus::Removed) => {
                    for mut entry in entries.into_iter().map(IntoActiveModel::into_active_model) {
                        entry.user_id = Set(Some(message.author.id.get() as i64));
                        entry.message_id = Set(Some(message.id.get() as i64));
                        entry.channel_id = Set(Some(message.channel_id.get() as i64));
                        entry.updated_at = Set(Utc::now().into());
                        entry.status = Set(status.unwrap());

                        let _ = entry.update(&ctx.data::<Data>().database.pool).await;
                    }
                }
                Some(WikiUrlStatus::Pending) | None => {
                    let mut embed = CreateEmbed::new().title("Warning").color(Color::ORANGE);

                    for status in WikiUrlStatus::iter() {
                        if let Some(formatted) = entries.format_for_embed(&status) {
                            let title = match status {
                                WikiUrlStatus::Added => "Link(s) already in the wiki:",
                                WikiUrlStatus::Pending => "Links(s) already in queue:",
                                WikiUrlStatus::Removed => {
                                    "Links(s) previously removed from the wiki:"
                                }
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
            }
        } else if let Some(status) = status {
            let _ = WikiUrls::insert_many(
                urls.into_iter()
                    .map(|url| wiki_urls::ActiveModel {
                        url: Set(url),
                        user_id: Set(Some(message.author.id.get() as i64)),
                        guild_id: Set(message.guild_id.map(|g| g.get() as i64)),
                        channel_id: Set(Some(message.channel_id.get() as i64)),
                        message_id: Set(Some(message.id.get() as i64)),
                        status: Set(status),
                        ..Default::default()
                    })
                    .collect::<Vec<_>>(),
            )
            .exec(&ctx.data::<Data>().database.pool)
            .await;
        }
    }
}
