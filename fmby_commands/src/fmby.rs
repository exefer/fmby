use crate::{Context, Error};
use poise::serenity_prelude::{ActivityData, OnlineStatus};

#[poise::command(
    slash_command,
    install_context = "Guild",
    interaction_context = "Guild",
    subcommands("status", "activity"),
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
    ctx.defer_ephemeral().await?;

    ctx.serenity_context().set_status(status.into());

    ctx.reply("Done!").await?;
    Ok(())
}

#[poise::command(slash_command, owners_only)]
pub async fn activity(ctx: Context<'_>, state: String) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    ctx.serenity_context()
        .set_activity(Some(ActivityData::custom(state)));

    ctx.reply("Done!").await?;
    Ok(())
}

#[must_use]
pub fn commands() -> [crate::Command; 1] {
    [fmby()]
}
