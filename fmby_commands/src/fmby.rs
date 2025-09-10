use crate::{Context, Error};
use fmby_entities::prelude::*;
use poise::{
    CreateReply,
    serenity_prelude::{ActivityData, CreateEmbed, OnlineStatus},
};
use sea_orm::prelude::*;

#[poise::command(
    slash_command,
    install_context = "Guild",
    interaction_context = "Guild",
    subcommands("status", "activity", "refresh_db"),
    subcommand_required
)]
pub async fn fmby(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[derive(Debug, poise::ChoiceParameter)]
pub enum OnlineStatusChoice {
    Online,
    Idle,
    DoNotDisturb,
    Invisible,
    Offline,
}

impl From<OnlineStatusChoice> for OnlineStatus {
    fn from(online_status: OnlineStatusChoice) -> Self {
        match online_status {
            OnlineStatusChoice::Online => OnlineStatus::Online,
            OnlineStatusChoice::Idle => OnlineStatus::Idle,
            OnlineStatusChoice::DoNotDisturb => OnlineStatus::DoNotDisturb,
            OnlineStatusChoice::Invisible => OnlineStatus::Invisible,
            OnlineStatusChoice::Offline => OnlineStatus::Offline,
        }
    }
}

/// Sets the bot's online status
#[poise::command(slash_command, owners_only)]
pub async fn status(ctx: Context<'_>, status: OnlineStatusChoice) -> Result<(), Error> {
    ctx.serenity_context().set_status(status.into());
    ctx.reply("Done!").await?;

    Ok(())
}

/// Sets the bot's activity to a custom message
#[poise::command(slash_command, owners_only)]
pub async fn activity(ctx: Context<'_>, state: String) -> Result<(), Error> {
    ctx.serenity_context()
        .set_activity(Some(ActivityData::custom(state)));
    ctx.reply("Done!").await?;

    Ok(())
}

/// Fetches wiki links and inserts them into the database, reporting the number of new rows
#[poise::command(slash_command, owners_only)]
pub async fn refresh_db(ctx: Context<'_>) -> Result<(), Error> {
    if let Ok(wiki_links) = fmby_core::utils::wiki::fetch_wiki_links().await {
        let pool = &ctx.data().database.pool;
        let before = WikiUrls::find().count(pool).await.unwrap_or(0);

        if let Err(e) = fmby_core::utils::wiki::insert_wiki_links(pool, wiki_links).await {
            ctx.reply(format!("{}", e)).await?;
        } else {
            let after = WikiUrls::find().count(pool).await.unwrap_or(0);
            ctx.reply(format!("Rows inserted: {}", after.saturating_sub(before)))
                .await?;
        }
    }

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
    limit: Option<usize>,
) -> Result<(), Error> {
    let result: String = fmby_core::utils::wiki::search_in_wiki(&query)
        .await
        .unwrap()
        .into_iter()
        .take(limit.unwrap_or(10))
        .map(|s| format!("- {}\n", s))
        .collect();

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

#[must_use]
pub fn commands() -> [crate::Command; 2] {
    [fmby(), search()]
}
