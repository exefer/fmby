use crate::{Context, Error};
use fmby_core::drama::fill_phrase;
use rand::prelude::*;

/// Generate funny piracy community drama
#[poise::command(slash_command)]
pub async fn drama(ctx: Context<'_>) -> Result<(), Error> {
    let drama_config = &ctx.data().drama_config;

    let filled = {
        let mut rng = rand::rng();
        let phrase = drama_config.phrases.choose(&mut rng).unwrap();
        fill_phrase(phrase, drama_config, &mut rng)
    };

    ctx.say(filled).await?;

    Ok(())
}

pub fn commands() -> [crate::Command; 1] {
    [drama()]
}
