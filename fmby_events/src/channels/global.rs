use fmby_core::{
    constants::{AUTO_THREAD_MAPPINGS, FmhyChannel},
    structs::Data,
    utils::{
        db::{get_wiki_urls_by_urls, infer_wiki_url_status, update_wiki_urls_with_message},
        formatters::UrlFormatter,
        url::extract_urls,
    },
};
use fmby_entities::{prelude::*, sea_orm_active_enums::WikiUrlStatus, wiki_urls};
use poise::serenity_prelude::{
    Channel, Color, CreateAllowedMentions, CreateEmbed, CreateMessage, CreateThread, Message,
    MessageReference, prelude::*,
};
use sea_orm::{ActiveValue::*, Iterable, prelude::*};

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

    let Some(urls) = extract_urls(&message.content) else {
        return;
    };

    let status = infer_wiki_url_status(message.channel_id.get());

    if let Some(entries) = get_wiki_urls_by_urls(&urls, &ctx.data::<Data>().database.pool).await {
        if !entries.is_empty() {
            match status {
                Some(WikiUrlStatus::Added) | Some(WikiUrlStatus::Removed) => {
                    update_wiki_urls_with_message(
                        entries,
                        message,
                        status.unwrap(),
                        &ctx.data::<Data>().database.pool,
                    )
                    .await;
                }
                Some(WikiUrlStatus::Pending) | None => {
                    if status.is_none() {
                        match message.channel(&ctx.http).await {
                            Ok(Channel::GuildThread(thread)) => {
                                if !matches!(
                                    thread.parent_id.get(),
                                    FmhyChannel::ADD_LINKS
                                        | FmhyChannel::NSFW_ADD_LINKS
                                        | FmhyChannel::LINK_TESTING
                                ) {
                                    return;
                                }
                            }
                            _ => return,
                        }
                    }

                    let mut embed = CreateEmbed::new().title("Warning").color(Color::ORANGE);

                    for status in WikiUrlStatus::iter() {
                        if let Some(formatted) = entries.format_for_embed(&status) {
                            let title = match status {
                                WikiUrlStatus::Added => "Links already in the wiki:",
                                WikiUrlStatus::Pending => "Links already in queue:",
                                WikiUrlStatus::Removed => "Links previously removed from the wiki:",
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
