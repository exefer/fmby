use poise::serenity_prelude::{small_fixed_array::FixedString, *};

pub async fn on_reaction_add(ctx: &Context, reaction: &Reaction) {
    let user = match reaction.user(&ctx.http).await {
        Ok(u) if !u.bot() => u,
        _ => return,
    };
    let message = match reaction.message(&ctx.http).await {
        Ok(m) => m,
        _ => return,
    };

    if reaction.emoji.unicode_eq("🔖")
        && let Some(guild_id) = reaction.guild_id
    {
        let avatar = message
            .author
            .avatar_url()
            .unwrap_or_else(|| message.author.default_avatar_url());

        let builder = CreateMessage::new().embed(
            CreateEmbed::new()
                .author(CreateEmbedAuthor::new(&message.author.name).icon_url(&avatar))
                .description(&message.content)
                .field(
                    "Jump",
                    format!("[Go to Message!]({})", message.link()),
                    false,
                )
                .footer(CreateEmbedFooter::new(format!(
                    "Guild: {} | Channel: #{}",
                    guild_id
                        .name(&ctx.cache)
                        .unwrap_or_else(|| "None".to_string()),
                    message
                        .guild_channel(&ctx.http)
                        .await
                        .map(|c| c.base.name.into_string())
                        .unwrap_or_else(|_| "None".to_string())
                )))
                .timestamp(Timestamp::now()),
        );

        if let Ok(m) = user.id.direct_message(&ctx.http, builder).await {
            let _ = m
                .react(
                    &ctx.http,
                    ReactionType::Unicode(FixedString::from_str_trunc("❌")),
                )
                .await;
        }
    }

    if reaction.emoji.unicode_eq("❌") && message.author.bot() && reaction.guild_id.is_none() {
        let _ = message.delete(&ctx.http, None).await;
    }
}
