use crate::{Context, Error};

/// Post the link to the bot's source code
#[poise::command(
    slash_command,
    prefix_command,
    category = "Meta",
    install_context = "Guild|User"
)]
pub async fn source(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("<https://github.com/exefer/fmby>").await?;

    Ok(())
}

/// Shutdown the bot gracefully
#[poise::command(prefix_command, owners_only, hide_in_help)]
pub async fn shutdown(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("**Bailing out, you are on your own. Good luck.**")
        .await?;
    ctx.framework().serenity_context.shutdown_all();

    Ok(())
}

/// Tells you how long the bot has been up for
#[poise::command(
    slash_command,
    prefix_command,
    category = "Meta",
    install_context = "Guild|User"
)]
pub async fn uptime(ctx: Context<'_>) -> Result<(), Error> {
    let uptime = ctx.data().time_started.elapsed();

    let calculation = |a, b| (a / b, a % b);

    let seconds = uptime.as_secs();
    let (minutes, seconds) = calculation(seconds, 60);
    let (hours, minutes) = calculation(minutes, 60);
    let (days, hours) = calculation(hours, 24);

    ctx.say(format!("`Uptime: {days}d {hours}h {minutes}m {seconds}s`"))
        .await?;

    Ok(())
}

#[poise::command(prefix_command, owners_only, hide_in_help)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;

    Ok(())
}

#[must_use]
pub fn commands() -> [crate::Command; 4] {
    [source(), shutdown(), uptime(), register()]
}
