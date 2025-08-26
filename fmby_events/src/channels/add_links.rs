use fmby_core::{
    config::AddLinksMessages,
    constants::DevChannel,
    structs::Data,
    utils::{db::WikiUrlFinder, formatters::UrlFormatter, url::extract_urls},
};
use fmby_entities::{prelude::*, sea_orm_active_enums::WikiUrlStatus, wiki_urls};
use fmby_migrations::OnConflict;
use poise::serenity_prelude::*;
use sea_orm::{ActiveValue::*, Iterable, TryInsertResult, prelude::*};

fn is_message_in_add_links(message: &Message) -> bool {
    message.channel_id.get() == DevChannel::AddLinks.id()
}

pub async fn on_message(ctx: &Context, message: &Message) {
    if !is_message_in_add_links(message) {
        return;
    };

    let Some(ref urls) = extract_urls(&message.content) else {
        return;
    };

    let wiki_entries = urls
        .find_wiki_url_entries(&ctx.data::<Data>().database.pool)
        .await;

    match wiki_entries {
        Ok(wiki_entries) if !wiki_entries.is_empty() => {
            let mut embed = CreateEmbed::new();

            for status in WikiUrlStatus::iter() {
                if let Some(formatted) = wiki_entries.format_for_embed(&status) {
                    let title = match status {
                        WikiUrlStatus::Added => {
                            AddLinksMessages::get_message_already_added(&ctx.data())
                        }
                        WikiUrlStatus::Pending => {
                            AddLinksMessages::get_message_already_pending(&ctx.data())
                        }
                        WikiUrlStatus::Removed => {
                            AddLinksMessages::get_message_previously_removed(&ctx.data())
                        }
                    };
                    embed = embed.field(title, formatted, false);
                }
            }

            let builder = CreateMessage::new()
                .add_embed(embed)
                .reference_message(MessageReference::from(message))
                .allowed_mentions(CreateAllowedMentions::new().replied_user(true));

            message.channel_id.send_message(&ctx.http, builder).await;
        }
        Ok(_) => {
            match WikiUrls::insert_many(
                urls.iter()
                    .map(|url| wiki_urls::ActiveModel {
                        url: Set(url.to_owned()),
                        user_id: Set(Some(message.author.id.get() as i64)),
                        guild_id: Set(message.guild_id.map(|g| g.get() as i64)),
                        channel_id: Set(Some(message.channel_id.get() as i64)),
                        message_id: Set(Some(message.id.get() as i64)),
                        status: Set(WikiUrlStatus::Pending),
                        ..Default::default()
                    })
                    .collect::<Vec<_>>(),
            )
            .on_conflict(
                OnConflict::column(wiki_urls::Column::Url)
                    .do_nothing()
                    .to_owned(),
            )
            .do_nothing()
            .exec(&ctx.data::<Data>().database.pool)
            .await
            {
                Ok(TryInsertResult::Inserted(insert_result)) => {}
                Err(_) => {}
                _ => {}
            };
        }
        Err(err) => {
            tracing::error!(
                error = %err,
                guild_id = ?message.guild_id.map(|g| g.get()),
                channel_id = %message.channel_id.get(),
                message_id = %message.id.get(),
                user_id = %message.author.id.get(),
                urls = ?urls,
                "Database error while handling add_links::on_message"
            );
        }
    }
}
