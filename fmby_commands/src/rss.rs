use crate::{Context, Error};
use fmby_core::rss::RssManager;
use fmby_entities::{prelude::*, rss_feeds, sea_orm_active_enums::RssFeedStatus};
use poise::{
    CreateReply,
    serenity_prelude::{self as serenity, CreateAllowedMentions},
};
use sea_orm::{
    ActiveValue::*, QueryFilter, QuerySelect, QueryTrait, prelude::*,
    sea_query::extension::postgres::PgExpr,
};
use url::Url;

async fn parse_uuid_or_reply(ctx: &Context<'_>, input: &str) -> Option<Uuid> {
    match input.parse::<u128>() {
        Ok(u) => Some(Uuid::from_u128(u)),
        Err(_) => {
            let _ = ctx
                .reply("Invalid input. Please choose from the autocompletion choices.")
                .await;
            None
        }
    }
}

async fn autocomplete_name<'a>(
    ctx: Context<'a>,
    partial: &'a str,
) -> serenity::CreateAutocompleteResponse<'a> {
    let feeds = RssFeeds::find()
        .apply_if((!partial.is_empty()).then_some(()), |query, _| {
            query.filter(Expr::col(rss_feeds::Column::Name).ilike(format!("%{}%", partial)))
        })
        .apply_if(ctx.guild_id().map(|g| g.get()), |query, guild_id| {
            query.filter(rss_feeds::Column::GuildId.eq(guild_id))
        })
        .limit(25)
        .all(&ctx.data().database.pool)
        .await
        .unwrap_or_default();

    let choices: Vec<_> = feeds
        .into_iter()
        .map(|feed| serenity::AutocompleteChoice::new(feed.name, feed.id.as_u128().to_string()))
        .collect();

    serenity::CreateAutocompleteResponse::new().set_choices(choices)
}

#[poise::command(
    slash_command,
    install_context = "Guild",
    interaction_context = "Guild",
    subcommands("add", "remove", "rename", "list"),
    subcommand_required
)]
pub async fn rss(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Add an RSS feed to the bot
#[poise::command(slash_command)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "Name of the RSS feed to add"] name: String,
    #[description = "URL of the RSS feed to add"] url: String,
) -> Result<(), Error> {
    match Url::parse(&url) {
        Ok(url) => {
            let rss_manager = RssManager::new(ctx.data().database.pool.clone().into());
            let feed = rss_feeds::ActiveModel {
                id: Set(Uuid::new_v4()),
                url: Set(url.to_string()),
                name: Set(name.clone()),
                channel_id: Set(ctx.channel_id().get() as i64),
                guild_id: Set(ctx.guild_id().unwrap().get() as i64),
                created_by: Set(ctx.author().id.get() as i64),
                status: Set(RssFeedStatus::Active),
                ..Default::default()
            };

            rss_manager.add_feed(feed).await?;

            ctx.reply(format!(
                "Successfully added \"{}\" RSS Feed with URL <{}>!",
                name, url
            ))
            .await?;
        }
        Err(_) => {
            ctx.reply("Unable to add RSS feed;  URL is not valid!")
                .await?;
        }
    }

    Ok(())
}

/// Remove an RSS feed from the bot (use autocompletion to select the feed)
#[poise::command(slash_command)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "Name of the RSS feed to remove"]
    #[autocomplete = "autocomplete_name"]
    name: String,
) -> Result<(), Error> {
    if let Some(uuid) = parse_uuid_or_reply(&ctx, &name).await {
        let feed = RssFeeds::delete_by_id(uuid)
            .exec_with_returning(&ctx.data().database.pool)
            .await?
            .into_iter()
            .next()
            .unwrap();

        ctx.reply(format!(
            "Successfully removed \"{}\" RSS Feed with URL <{}>!",
            feed.name, feed.url
        ))
        .await?;
    }

    Ok(())
}

/// Rename an RSS feed from the bot (use autocompletion to select the feed)
#[poise::command(slash_command)]
pub async fn rename(
    ctx: Context<'_>,
    #[description = "Name of the RSS feed to rename"]
    #[autocomplete = "autocomplete_name"]
    name: String,
    #[description = "The new name for the RSS feed"] new_name: String,
) -> Result<(), Error> {
    if let Some(uuid) = parse_uuid_or_reply(&ctx, &name).await {
        let feed = RssFeeds::update_many()
            .col_expr(rss_feeds::Column::Name, Expr::value(new_name))
            .filter(rss_feeds::Column::Id.eq(uuid))
            .exec_with_returning(&ctx.data().database.pool)
            .await?
            .into_iter()
            .next()
            .unwrap();

        ctx.reply(format!(
            "Successfully renamed to \"{}\" RSS Feed with URL <{}>!",
            feed.name, feed.url
        ))
        .await?;
    }

    Ok(())
}

/// Lists the RSS feeds the bot is subscribed to in the current channel
#[poise::command(slash_command)]
pub async fn list(
    ctx: Context<'_>,
    #[description = "Whether the response should only be visible to you"] ephemeral: Option<bool>,
) -> Result<(), Error> {
    let feeds = RssFeeds::find()
        .filter(rss_feeds::Column::ChannelId.eq(ctx.channel_id().get()))
        .apply_if(ctx.guild_id().map(|g| g.get()), |query, guild_id| {
            query.filter(rss_feeds::Column::GuildId.eq(guild_id))
        })
        .all(&ctx.data().database.pool)
        .await?;

    let content = if feeds.is_empty() {
        "There are no RSS feed subscriptions in this channel.".to_string()
    } else {
        feeds
            .into_iter()
            .map(|feed| {
                format!(
                    "- {}: <{}> (added by <@{}>)",
                    feed.name, feed.url, feed.created_by
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    ctx.send(
        CreateReply::new()
            .content(content)
            .reply(true)
            .ephemeral(ephemeral.unwrap_or(true))
            .allowed_mentions(CreateAllowedMentions::new().all_users(false)),
    )
    .await?;

    Ok(())
}

#[poise::command(prefix_command)]
pub async fn fetch_feed_title(ctx: Context<'_>, url: String) -> Result<(), Error> {
    let fetcher = fmby_core::rss::RssFetcher::new(&fmby_core::rss::RssConfig::default());
    let message = ctx.reply("Fetching RSS feed...").await?;

    let content = match fetcher.validate_feed_url(&url).await {
        Ok(title) => format!("The feed title is: {}", title),
        Err(e) => e.to_string(),
    };

    message
        .edit(ctx, CreateReply::new().content(content))
        .await?;

    Ok(())
}

#[must_use]
pub fn commands() -> [crate::Command; 2] {
    [rss(), fetch_feed_title()]
}
