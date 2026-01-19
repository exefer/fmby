use crate::{Context, Error};
use fmby_core::{
    constants::{FMHY_SINGLE_PAGE_ENDPOINT, FmhyChannel},
    utils::{
        db::{ChunkSize, infer_wiki_url_status},
        message::get_content_or_referenced,
        url::{clean_url, extract_urls},
        wiki::collect_wiki_urls,
    },
};
use fmby_entities::{prelude::*, sea_orm_active_enums::WikiUrlStatus, wiki_urls};
use poise::{
    CreateReply,
    serenity_prelude::{
        ActivityData, AutocompleteChoice, Color, CreateAutocompleteResponse, CreateEmbed,
        CreateEmbedFooter, CreateMessage, EditMessage, GenericChannelId, OnlineStatus,
        futures::StreamExt,
    },
};
use sea_orm::{
    ActiveValue::*,
    QueryOrder, QuerySelect, QueryTrait, TransactionTrait,
    prelude::*,
    sea_query::{OnConflict, extension::postgres::PgExpr},
    sqlx::types::chrono::Utc,
};
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

/// Migrate existing messages from designated channels into the wiki database
// by extracting URLs, determining their status (pending, added, removed),
// and storing them with associated metadata.
#[poise::command(slash_command, owners_only)]
pub async fn migrate(
    ctx: Context<'_>,
    #[description = "Whether to only process wiki links without scanning message history"]
    only_wiki: bool,
) -> Result<(), Error> {
    let start = std::time::Instant::now();
    let content = reqwest::get(FMHY_SINGLE_PAGE_ENDPOINT)
        .await?
        .text()
        .await?;
    let urls = collect_wiki_urls(&content)
        .iter()
        .map(|url| clean_url(url).to_owned())
        .collect::<Vec<_>>();
    drop(content);
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

    if !only_wiki {
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

                let Some(m_content) = get_content_or_referenced(&message) else {
                    continue;
                };

                let Some(m_urls) = extract_urls(m_content) else {
                    messages_skipped += 1;
                    continue;
                };

                let urls = match status {
                    WikiUrlStatus::Pending => m_urls,
                    WikiUrlStatus::Added => {
                        let urls_in_wiki = m_urls
                            .into_iter()
                            .filter(|url| urls.contains(url))
                            .collect::<Vec<_>>();

                        if urls_in_wiki.is_empty() {
                            continue;
                        }

                        urls_in_wiki
                    }
                    WikiUrlStatus::Removed => {
                        let urls_not_in_wiki = m_urls
                            .into_iter()
                            .filter(|url| !urls.contains(url))
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

            let process_rate = if messages_processed > 0 {
                100.0 * (messages_processed - messages_skipped) as f64 / messages_processed as f64
            } else {
                0.0
            };
            let next_channel_id = channel_ids.get(i + 1);

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
                                    next_channel_id.map_or_else(
                                        || "None".to_owned(),
                                        |id| format!("<#{}>", id),
                                    ),
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
    }

    for url in urls {
        entries
            .entry(url.clone())
            .or_insert_with(|| wiki_urls::ActiveModel {
                url: Set(url),
                guild_id: Set(ctx.guild_id().map(|g| g.get() as i64)),
                created_at: Set(Utc::now().into()),
                updated_at: Set(Utc::now().into()),
                status: Set(WikiUrlStatus::Added),
                ..Default::default()
            });
    }

    if !only_wiki {
        reply
            .edit(ctx.http(), EditMessage::new().content(""))
            .await?;
    }

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

/// Search for query in the wiki
#[poise::command(slash_command)]
pub async fn search(
    ctx: Context<'_>,
    #[description = "The term or phrase you want to search for in the wiki"] query: String,
    #[description = "The maximum number of search results to return (default is 10)"]
    #[min = 1]
    #[max = 25]
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

async fn autocomplete_url<'a>(
    ctx: Context<'a>,
    partial: &'a str,
) -> CreateAutocompleteResponse<'a> {
    let urls: Vec<String> = WikiUrls::find()
        .select_only()
        .column(wiki_urls::Column::Url)
        .apply_if((!partial.is_empty()).then_some(()), |query, _| {
            query.filter(
                Expr::col(wiki_urls::Column::Url).ilike(format!("%{}%", clean_url(partial))),
            )
        })
        .limit(25)
        .order_by_asc(wiki_urls::Column::Url)
        .into_tuple()
        .all(&ctx.data().database.pool)
        .await
        .unwrap_or_default();

    let choices: Vec<_> = urls.into_iter().map(AutocompleteChoice::from).collect();

    CreateAutocompleteResponse::new().set_choices(choices)
}

/// Displays context information about a specific wiki URL
#[poise::command(slash_command)]
pub async fn context(
    ctx: Context<'_>,
    #[description = "The URL to retrieve context information for"]
    #[autocomplete = "autocomplete_url"]
    url: String,
    #[description = "Whether the response should only be visible to you"] ephemeral: Option<bool>,
) -> Result<(), Error> {
    if let Some(entry) = WikiUrls::find()
        .filter(wiki_urls::Column::Url.eq(url))
        .one(&ctx.data().database.pool)
        .await?
    {
        let context = match (entry.guild_id, entry.channel_id, entry.message_id) {
            (Some(guild_id), Some(channel_id), Some(message_id)) => {
                format!(
                    "https://discord.com/channels/{}/{}/{}",
                    guild_id, channel_id, message_id
                )
            }
            _ => "Unavailable".to_owned(),
        };

        ctx.send(
            CreateReply::new()
                .embed(
                    CreateEmbed::new()
                        .field("URL", entry.url, false)
                        .field(
                            "Updated By",
                            entry.user_id.map_or_else(
                                || "Unavailable".to_owned(),
                                |id| format!("<@{}>", id),
                            ),
                            false,
                        )
                        .field("Context", context, false)
                        .field(
                            "Status",
                            match entry.status {
                                WikiUrlStatus::Pending => "Pending",
                                WikiUrlStatus::Added => "Added",
                                WikiUrlStatus::Removed => "Removed",
                            },
                            false,
                        )
                        .field(
                            "Created",
                            format!("<t:{}:R>", entry.created_at.to_utc().timestamp()),
                            true,
                        )
                        .field(
                            "Updated",
                            format!("<t:{}:R>", entry.updated_at.to_utc().timestamp()),
                            true,
                        )
                        .color(match entry.status {
                            WikiUrlStatus::Pending => Color::ORANGE,
                            WikiUrlStatus::Added => Color::DARK_GREEN,
                            WikiUrlStatus::Removed => Color::RED,
                        }),
                )
                .ephemeral(ephemeral.unwrap_or(true)),
        )
        .await?;
    } else {
        ctx.send(
            CreateReply::new()
                .content("Invalid input. Please choose from the autocompletion choices.")
                .ephemeral(true),
        )
        .await?;
    }

    Ok(())
}

#[poise::command(prefix_command, owners_only, aliases("incons"))]
pub async fn inconsistencies(ctx: Context<'_>) -> Result<(), Error> {
    let entries = WikiUrls::find()
        .filter(wiki_urls::Column::ChannelId.is_not_in([
            FmhyChannel::NSFW_ADD_LINKS,
            FmhyChannel::NSFW_RECENTLY_ADDED,
            FmhyChannel::NSFW_REMOVED,
        ]))
        .order_by_desc(wiki_urls::Column::UpdatedAt)
        .all(&ctx.data().database.pool)
        .await?;
    let content = reqwest::get(FMHY_SINGLE_PAGE_ENDPOINT)
        .await?
        .text()
        .await?;

    let urls = collect_wiki_urls(&content)
        .iter()
        .map(|url| clean_url(url).to_owned())
        .collect::<Vec<_>>();

    let mut added_not_in_wiki = Vec::new();
    let mut in_wiki_not_added = Vec::new();

    for entry in entries {
        let in_wiki = urls.contains(&entry.url);

        match entry.status {
            WikiUrlStatus::Added => {
                if !in_wiki {
                    added_not_in_wiki.push(entry.url);
                }
            }
            WikiUrlStatus::Removed | WikiUrlStatus::Pending => {
                if in_wiki {
                    in_wiki_not_added.push(entry.url);
                }
            }
        }
    }

    let mut embeds = Vec::new();

    for chunk in added_not_in_wiki.chunks(50) {
        let description = chunk
            .iter()
            .map(|url| format!("- {}", url))
            .collect::<Vec<_>>()
            .join("\n");

        embeds.push(
            CreateEmbed::new()
                .title("Added → Removed")
                .description(description)
                .color(Color::RED),
        );
    }

    for chunk in in_wiki_not_added.chunks(50) {
        let description = chunk
            .iter()
            .map(|url| format!("- {}", url))
            .collect::<Vec<_>>()
            .join("\n");

        embeds.push(
            CreateEmbed::new()
                .title("Removed/Pending → Added")
                .description(description)
                .color(Color::ORANGE),
        );
    }

    if embeds.is_empty() {
        ctx.say("No inconsistencies found.").await?;
    } else {
        for embed in embeds {
            ctx.channel_id()
                .send_message(ctx.http(), CreateMessage::new().embed(embed))
                .await?;
        }
    }

    Ok(())
}

#[must_use]
pub fn commands() -> [crate::Command; 4] {
    [fmby(), search(), context(), inconsistencies()]
}
