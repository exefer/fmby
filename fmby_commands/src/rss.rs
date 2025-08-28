use crate::{Context, Error};
use fmby_core::rss::RssManager;
use fmby_entities::{prelude::*, rss_feeds, sea_orm_active_enums::RssFeedStatus};
use poise::serenity_prelude as serenity;
use sea_orm::{
    ActiveValue::*, QueryFilter, QuerySelect, QueryTrait, prelude::*,
    sea_query::extension::postgres::PgExpr,
};
use url::Url;

async fn autocomplete_name<'a>(
    ctx: Context<'a>,
    partial: &'a str,
) -> serenity::CreateAutocompleteResponse<'a> {
    let feeds = RssFeeds::find()
        .apply_if(Option::from(partial.is_empty()), |query, _| {
            query.filter(Expr::col(rss_feeds::Column::Name).ilike(format!("%{}%", partial)))
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
    subcommands("add", "remove"),
    subcommand_required
)]
pub async fn rss(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, owners_only)]
pub async fn add(ctx: Context<'_>, url: String, name: String) -> Result<(), Error> {
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

#[poise::command(slash_command, owners_only)]
pub async fn remove(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_name"] name: String,
) -> Result<(), Error> {
    match name.parse::<u128>() {
        Ok(uuid) => {
            let uuid = Uuid::from_u128(uuid);
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
        Err(_) => {
            ctx.reply("Invalid input. Please choose from the autocompletion choices.")
                .await?;
        }
    }

    Ok(())
}

#[poise::command(prefix_command, owners_only)]
pub async fn fetch_feed_title(ctx: Context<'_>, url: String) -> Result<(), Error> {
    let fetcher = fmby_core::rss::RssFetcher::new(&fmby_core::rss::RssConfig::default());
    let message = ctx.reply("Fetching RSS feed...").await?;

    let content = match fetcher.validate_feed_url(&url).await {
        Ok(title) => format!("The feed title is: {}", title),
        Err(e) => e.to_string(),
    };

    message
        .edit(ctx, poise::CreateReply::new().content(content))
        .await?;

    Ok(())
}

#[must_use]
pub fn commands() -> [crate::Command; 2] {
    [rss(), fetch_feed_title()]
}
