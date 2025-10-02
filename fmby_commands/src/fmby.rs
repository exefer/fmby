use crate::{Context, Error};
use fmby_core::{
    constants::{FMHY_SINGLE_PAGE_ENDPOINT, FmhyChannel},
    utils::{
        db::{ChunkSize, infer_wiki_url_status, update_wiki_urls_with_message},
        url::extract_urls,
    },
};
use fmby_entities::{prelude::*, sea_orm_active_enums::WikiUrlStatus, wiki_urls};
use poise::{
    CreateReply,
    serenity_prelude::{
        ActivityData, Channel, CreateEmbed, CreateEmbedFooter, EditMessage, GenericChannelId,
        Message, OnlineStatus, futures::StreamExt,
    },
};
use sea_orm::{ActiveValue::*, TransactionTrait, prelude::*, sea_query::OnConflict};
use std::collections::HashMap;

#[poise::command(
    slash_command,
    install_context = "Guild",
    interaction_context = "Guild",
    subcommands("status", "activity", "migrate"),
    subcommand_required
)]
pub async fn fmby(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[derive(poise::ChoiceParameter)]
pub enum OnlineStatusChoice {
    Online,
    Idle,
    DoNotDisturb,
    Invisible,
    Offline,
}

/// Sets the bot's online status
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn status(ctx: Context<'_>, status: OnlineStatusChoice) -> Result<(), Error> {
    let status = match status {
        OnlineStatusChoice::Online => OnlineStatus::Online,
        OnlineStatusChoice::Idle => OnlineStatus::Idle,
        OnlineStatusChoice::DoNotDisturb => OnlineStatus::DoNotDisturb,
        OnlineStatusChoice::Invisible => OnlineStatus::Invisible,
        OnlineStatusChoice::Offline => OnlineStatus::Offline,
    };

    ctx.serenity_context().set_status(status);
    ctx.reply("Done!").await?;

    Ok(())
}

#[derive(poise::ChoiceParameter)]
pub enum ActivityTypeChoice {
    Playing,
    Listening,
    Streaming,
    Watching,
    Competing,
    Custom,
}

/// Sets the bot's activity
#[poise::command(slash_command, required_permissions = "ADMINISTRATOR")]
pub async fn activity(
    ctx: Context<'_>,
    #[description = "Type of activity to display"] activity_type: ActivityTypeChoice,
    #[description = "Text describing the activity"] text: String,
    #[description = "Extra information (like streaming URL) if required"] extra: Option<String>,
) -> Result<(), Error> {
    let activity = match activity_type {
        ActivityTypeChoice::Playing => ActivityData::playing(&text),
        ActivityTypeChoice::Listening => ActivityData::listening(&text),
        ActivityTypeChoice::Streaming => {
            let Some(url) = extra else {
                ctx.reply("You must provide a URL when streaming.").await?;
                return Ok(());
            };
            ActivityData::streaming(&text, url)?
        }
        ActivityTypeChoice::Watching => ActivityData::watching(&text),
        ActivityTypeChoice::Competing => ActivityData::competing(&text),
        ActivityTypeChoice::Custom => ActivityData::custom(&text),
    };

    ctx.serenity_context().set_activity(Some(activity));
    ctx.reply("Done!").await?;

    Ok(())
}

/// Search for query in the wiki
#[poise::command(slash_command)]
pub async fn search(
    ctx: Context<'_>,
    #[description = "The term or phrase you want to search for in the wiki"] query: String,
    #[min = 1]
    #[max = 25]
    #[description = "The maximum number of search results to return (default is 10)"]
    limit: Option<u8>,
) -> Result<(), Error> {
    let result: String = fmby_core::utils::wiki::search_in_wiki(&query)
        .await
        .unwrap()
        .into_iter()
        .take(limit.unwrap_or(10) as usize)
        .map(|s| format!("- {}", s))
        .collect::<Vec<_>>()
        .join("\n");

    ctx.send(
        CreateReply::new().embed(
            CreateEmbed::new()
                .title(format!("Search results for \"{}\"", query))
                .description(if result.is_empty() {
                    "Nothing found."
                } else {
                    &result
                }),
        ),
    )
    .await?;

    Ok(())
}

/// Migrate existing messages from designated channels into the wiki database
// by extracting URLs, determining their status (pending, added, removed),
// and storing them with associated metadata.
#[poise::command(slash_command, owners_only)]
pub async fn migrate(ctx: Context<'_>) -> Result<(), Error> {
    let start = std::time::Instant::now();
    let content = reqwest::get(FMHY_SINGLE_PAGE_ENDPOINT)
        .await?
        .text()
        .await?;
    let mut messages_processed = 0u32;
    let mut messages_skipped = 0u32;
    let mut urls_processed = 0u32;
    let mut entries = HashMap::new();
    let channel_ids = [
        FmhyChannel::RECENTLY_ADDED,
        FmhyChannel::NSFW_RECENTLY_ADDED,
        FmhyChannel::ADD_LINKS,
        FmhyChannel::NSFW_ADD_LINKS,
        FmhyChannel::DEAD_SITES,
        FmhyChannel::REMOVE_SITES,
        FmhyChannel::NSFW_REMOVED,
    ];
    ctx.say("Starting migration...").await?;
    let mut reply = ctx.channel_id().say(ctx.http(), "Processing...").await?;

    for (i, &channel_id) in channel_ids.iter().enumerate() {
        let mut messages = GenericChannelId::new(channel_id)
            .messages_iter(ctx.http())
            .boxed();

        let Some(status) = infer_wiki_url_status(channel_id) else {
            continue;
        };

        while let Some(Ok(message)) = messages.next().await {
            if message.author.bot() {
                continue;
            }

            messages_processed += 1;

            let Some(urls) = extract_urls(&message.content) else {
                messages_skipped += 1;
                continue;
            };

            let urls = match status {
                WikiUrlStatus::Pending => urls,
                WikiUrlStatus::Added => {
                    let urls_in_wiki = urls
                        .into_iter()
                        .filter(|url| content.contains(url))
                        .collect::<Vec<_>>();

                    if urls_in_wiki.is_empty() {
                        continue;
                    }

                    urls_in_wiki
                }
                WikiUrlStatus::Removed => {
                    let urls_not_in_wiki = urls
                        .into_iter()
                        .filter(|url| !content.contains(url))
                        .collect::<Vec<_>>();

                    if urls_not_in_wiki.is_empty() {
                        continue;
                    }

                    urls_not_in_wiki
                }
            };

            urls_processed += urls.len() as u32;

            for url in urls {
                entries
                    .entry(url.clone())
                    .or_insert_with(|| wiki_urls::ActiveModel {
                        url: Set(url),
                        user_id: Set(Some(message.author.id.get() as i64)),
                        message_id: Set(Some(message.id.get() as i64)),
                        channel_id: Set(Some(message.channel_id.get() as i64)),
                        guild_id: Set(ctx.guild_id().map(|g| g.get() as i64)),
                        created_at: Set(message.timestamp.fixed_offset()),
                        updated_at: Set(message.timestamp.fixed_offset()),
                        status: Set(status),
                        ..Default::default()
                    });
            }
        }

        let next_channel_id = channel_ids.get(i + 1);
        let process_rate = if messages_processed > 0 {
            100.0 * (messages_processed - messages_skipped) as f64 / messages_processed as f64
        } else {
            0.0
        };

        reply
            .edit(
                ctx.http(),
                EditMessage::new().embed(
                    CreateEmbed::new()
                        .title("Migration Progress")
                        .fields([
                            (
                                "Messages",
                                format!(
                                    "Processed: {}\nSkipped: {}\nProcess rate: {:.1}% ({})",
                                    messages_processed,
                                    messages_skipped,
                                    process_rate,
                                    messages_processed + messages_skipped
                                ),
                                false,
                            ),
                            ("URLs processed", urls_processed.to_string(), false),
                            ("Current channel", format!("<#{}>", channel_id), false),
                            (
                                "Next channel",
                                next_channel_id
                                    .map(|id| format!("<#{}>", id))
                                    .unwrap_or_else(|| "None".to_string()),
                                false,
                            ),
                            ("Time elapsed", format!("{:.2?}", start.elapsed()), false),
                        ])
                        .footer(CreateEmbedFooter::new(
                            "Progress is updated after each channel",
                        )),
                ),
            )
            .await?;
    }

    reply
        .edit(ctx.http(), EditMessage::new().content(""))
        .await?;

    let mut entries: Vec<_> = entries.into_values().collect();
    let chunk_size = WikiUrls::chunk_size();

    while !entries.is_empty() {
        let chunk: Vec<_> = entries.drain(..chunk_size.min(entries.len())).collect();
        let txn = ctx.data().database.pool.begin().await?;

        let _ = WikiUrls::insert_many(chunk)
            .on_conflict(
                OnConflict::column(wiki_urls::Column::Url)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(&txn)
            .await;

        txn.commit().await?;
    }

    ctx.channel_id()
        .say(ctx.http(), "Migration completed.")
        .await?;

    Ok(())
}

#[poise::command(context_menu_command = "Update entries", owners_only)]
pub async fn update_entries_in_message(ctx: Context<'_>, message: Message) -> Result<(), Error> {
    let Some(urls) = extract_urls(&message.content) else {
        return Ok(());
    };

    let mut status = infer_wiki_url_status(message.channel_id.get());

    if status.is_none() {
        match message.channel(ctx.http()).await {
            Ok(Channel::GuildThread(thread)) => {
                if !matches!(
                    thread.parent_id.get(),
                    FmhyChannel::ADD_LINKS
                        | FmhyChannel::NSFW_ADD_LINKS
                        | FmhyChannel::LINK_TESTING
                ) {
                    return Ok(());
                }
                status = Some(WikiUrlStatus::Pending);
            }
            _ => return Ok(()),
        }
    }

    let status = status.unwrap();
    let entries = WikiUrls::find()
        .filter(wiki_urls::Column::Url.is_in(&urls))
        .all(&ctx.data().database.pool)
        .await?;

    update_wiki_urls_with_message(entries, &message, status, &ctx.data().database.pool).await;

    Ok(())
}

#[poise::command(context_menu_command = "Delete entries", owners_only)]
pub async fn delete_entries_in_message(ctx: Context<'_>, message: Message) -> Result<(), Error> {
    let Some(urls) = extract_urls(&message.content) else {
        return Ok(());
    };

    let _ = WikiUrls::delete_many()
        .filter(wiki_urls::Column::Url.is_in(urls))
        .exec(&ctx.data().database.pool)
        .await;

    Ok(())
}

#[must_use]
pub fn commands() -> [crate::Command; 4] {
    [
        fmby(),
        search(),
        update_entries_in_message(),
        delete_entries_in_message(),
    ]
}
