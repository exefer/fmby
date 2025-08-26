use crate::{Context, Error};
use poise::serenity_prelude::{ActivityData, OnlineStatus};
use sea_orm::{EntityTrait, PaginatorTrait};

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

#[poise::command(slash_command, owners_only)]
pub async fn status(ctx: Context<'_>, status: OnlineStatusChoice) -> Result<(), Error> {
    ctx.serenity_context().set_status(status.into());
    ctx.reply("Done!").await?;

    Ok(())
}

#[poise::command(slash_command, owners_only)]
pub async fn activity(ctx: Context<'_>, state: String) -> Result<(), Error> {
    ctx.serenity_context()
        .set_activity(Some(ActivityData::custom(state)));
    ctx.reply("Done!").await?;

    Ok(())
}

#[poise::command(slash_command, owners_only)]
pub async fn refresh_db(ctx: Context<'_>) -> Result<(), Error> {
    use fmby_core::utils::url::{fetch_wiki_links, insert_wiki_urls};
    use fmby_entities::prelude::*;

    if let Ok(wiki_links) = fetch_wiki_links().await {
        let pool = &ctx.data().database.pool;
        let before = WikiUrls::find().count(pool).await.unwrap_or(0);

        if let Err(err) = insert_wiki_urls(pool, wiki_links).await {
            ctx.reply(format!("{}", err)).await?;
        } else {
            let after = WikiUrls::find().count(pool).await.unwrap_or(0);
            ctx.reply(format!("Rows inserted: {}", after.saturating_sub(before)))
                .await?;
        }
    }

    Ok(())
}

#[must_use]
pub fn commands() -> [crate::Command; 1] {
    [fmby()]
}
