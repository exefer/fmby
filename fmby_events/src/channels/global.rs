use fmby_core::{
    constants::{AUTO_THREAD_MAPPINGS, FmhyChannel, FmhyServerRole},
    structs::Data,
    utils::{
        db::{get_wiki_urls_by_urls, infer_wiki_url_status, update_wiki_urls_with_message},
        formatters::UrlFormatter,
        url::extract_urls,
    },
};
use fmby_entities::{prelude::*, sea_orm_active_enums::WikiUrlStatus, wiki_urls};
use poise::serenity_prelude::{
    Channel, Color, CreateAllowedMentions, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter,
    CreateMessage, CreateThread, Message, MessageReference, Reaction, ReactionType, Timestamp,
    prelude::*, small_fixed_array::FixedString,
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

    let message = if message.content.is_empty() {
        if let Some(referenced) = &message.referenced_message {
            referenced
        } else {
            return;
        }
    } else {
        message
    };

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
                                ) || thread.total_message_sent == 0
                                {
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

                    if let Ok(m) = message
                        .channel_id
                        .send_message(
                            &ctx.http,
                            CreateMessage::new()
                                .add_embed(embed)
                                .reference_message(MessageReference::from(message))
                                .allowed_mentions(CreateAllowedMentions::new().replied_user(true)),
                        )
                        .await
                    {
                        let _ = m
                            .react(
                                &ctx.http,
                                ReactionType::Unicode(FixedString::from_str_trunc("❌")),
                            )
                            .await;
                    };
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

pub async fn on_reaction_add(ctx: &Context, reaction: &Reaction) {
    let (Ok(user), Ok(message)) = (
        reaction.user(&ctx.http).await,
        reaction.message(&ctx.http).await,
    ) else {
        return;
    };

    if reaction.emoji.unicode_eq("🔖")
        && let Some(guild_id) = reaction.guild_id
        && let Ok(m) = user
            .id
            .direct_message(
                &ctx.http,
                CreateMessage::new().embed(
                    CreateEmbed::new()
                        .author(
                            CreateEmbedAuthor::new(&message.author.name).icon_url(
                                message
                                    .author
                                    .avatar_url()
                                    .unwrap_or_else(|| message.author.default_avatar_url()),
                            ),
                        )
                        .description(&message.content)
                        .field(
                            "Jump",
                            format!("[Go to Message!]({})", message.link()),
                            false,
                        )
                        .footer(CreateEmbedFooter::new(format!(
                            "Guild: {} | Channel: #{}",
                            guild_id
                                .name(&ctx.cache)
                                .unwrap_or_else(|| "None".to_string()),
                            message
                                .guild_channel(&ctx.http)
                                .await
                                .map(|c| c.base.name.into_string())
                                .unwrap_or_else(|_| "None".to_string())
                        )))
                        .timestamp(Timestamp::now()),
                ),
            )
            .await
    {
        let _ = m
            .react(
                &ctx.http,
                ReactionType::Unicode(FixedString::from_str_trunc("❌")),
            )
            .await;
    }

    if reaction.emoji.unicode_eq("❌") && message.author.bot() && reaction.guild_id.is_none() {
        let _ = message.delete(&ctx.http, None).await;
    }

    if reaction.emoji.unicode_eq("❌")
        && !user.bot()
        && message.author.bot()
        && let Some(member) = reaction.member.as_ref()
        && message.reactions.iter().any(|m| {
            m.me && m.reaction_type == ReactionType::Unicode(FixedString::from_str_trunc("❌"))
        })
        && (member.roles.iter().any(|r| {
            matches!(
                r.get(),
                FmhyServerRole::FIRST_MATE | FmhyServerRole::CELESTIAL | FmhyServerRole::CAPTAIN
            )
        }) || message
            .referenced_message
            .as_ref()
            .is_some_and(|m| user.id == m.author.id))
    {
        let _ = message.delete(&ctx.http, None).await;

        if let Some(m) = message.referenced_message {
            let _ = m.delete(&ctx.http, None).await;
        }
    }
}
