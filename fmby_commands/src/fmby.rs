use crate::{Context, Error};
use poise::serenity_prelude::ActivityData;

#[poise::command(
    slash_command,
    install_context = "Guild",
    interaction_context = "Guild",
    subcommands("set_name", "set_activity"),
    subcommand_required
)]
pub async fn fmby(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command)]
pub async fn set_name(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command)]
pub async fn set_activity(ctx: Context<'_>, state: String) -> Result<(), Error> {
    ctx.serenity_context()
        .set_activity(Some(ActivityData::custom(state)));

    Ok(())
}

#[must_use]
pub fn commands() -> [crate::Command; 1] {
    [fmby()]
}
