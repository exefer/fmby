use crate::{Context, Error};
use fmby_core::drama::fill_phrase;
use image::ImageFormat;
use poise::{
    CreateReply,
    serenity_prelude::{CreateAttachment, GetMessages},
};
use rand::prelude::*;
use regex::Regex;
use std::{collections::HashMap, io::Cursor, sync::LazyLock};
use wordcloud_rs::{Token, WordCloud};

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

static TOKEN_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\w+").unwrap());

fn tokenize(text: &str) -> Vec<(Token, f32)> {
    let mut counts: HashMap<String, usize> = HashMap::new();

    for token in TOKEN_RE.find_iter(text) {
        *counts.entry(token.as_str().to_owned()).or_default() += 1;
    }

    counts
        .into_iter()
        .map(|(k, v)| (Token::Text(k), v as f32))
        .collect()
}

/// Generates a wordcloud from the channel's message history
#[poise::command(slash_command)]
pub async fn wordcloud(
    ctx: Context<'_>,
    #[min = 1]
    #[max = 100]
    #[description = "The number of recent messages to include when generating the wordcloud (default is 25)"]
    message_limit: Option<u8>,
) -> Result<(), Error> {
    let tokens = ctx
        .channel_id()
        .messages(ctx, GetMessages::new().limit(message_limit.unwrap_or(25)))
        .await?
        .into_iter()
        .flat_map(|m| tokenize(&m.content_safe(ctx.cache())))
        .collect::<Vec<_>>();
    let wc = WordCloud::new().generate(tokens);

    let mut buf = Cursor::new(Vec::new());
    wc.write_to(&mut buf, ImageFormat::Png)?;

    ctx.send(
        CreateReply::new().attachment(CreateAttachment::bytes(buf.into_inner(), "wordcloud.png")),
    )
    .await?;

    Ok(())
}

pub fn commands() -> [crate::Command; 2] {
    [drama(), wordcloud()]
}
