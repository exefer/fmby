use serenity::all::*;

pub struct BookmarkHandler;

#[async_trait]
impl EventHandler for BookmarkHandler {
    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        let user = match reaction.user(&ctx.http).await {
            Ok(u) if !u.bot => u,
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
            match user
                .direct_message(
                    &ctx.http,
                    CreateMessage::new().embed(
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
                                guild_id.name(&ctx.cache).unwrap(),
                                message.channel_id.name(&ctx.http).await.unwrap()
                            )))
                            .timestamp(Timestamp::now()),
                    ),
                )
                .await
            {
                Ok(m) => {
                    let _ = m.react(&ctx.http, ReactionType::Unicode("❌".into())).await;
                }
                Err(_) => todo!(),
            }
        }

        if reaction.emoji.unicode_eq("❌") && message.author.bot && reaction.guild_id.is_none() {
            let _ = message.delete(&ctx.http).await;
        }
    }
}
