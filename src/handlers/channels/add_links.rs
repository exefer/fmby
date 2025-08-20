use crate::{
    config::AddLinksMessages,
    constants::DevChannel,
    shared::FmbyDatabase,
    utils::{db::WikiUrlFinder, formatters::UrlFormatter, url::extract_urls},
};
use entity::{
    sea_orm_active_enums::WikiUrlStatus,
    wiki_urls::{self, Entity as WikiUrls},
};
use futures::future::BoxFuture;
use migration::OnConflict;
use sea_orm::{ActiveValue::*, Iterable, sea_query};
use sea_orm::{TryInsertResult, prelude::*};
use serenity::all::*;

pub struct AddLinksHandler;

impl AddLinksHandler {
    fn is_message_in_add_links(message: &Message) -> bool {
        message.channel_id.get() == DevChannel::AddLinks.id()
    }
}

#[async_trait]
impl EventHandler for AddLinksHandler {
    async fn message(&self, ctx: Context, message: Message) {
        if !Self::is_message_in_add_links(&message) {
            return;
        };

        let Some(ref urls) = extract_urls(&message.content) else {
            return;
        };

        let wiki_entries = FmbyDatabase::with_connection(&ctx.data, |conn| async move {
            urls.find_wiki_url_entries(&conn).await
        })
        .await;

        match wiki_entries {
            Ok(wiki_entries) if !wiki_entries.is_empty() => {
                let mut embed = CreateEmbed::new();

                for status in WikiUrlStatus::iter() {
                    if let Some(formatted) = wiki_entries.format_for_embed(&status) {
                        let title = match status {
                            WikiUrlStatus::Added => {
                                AddLinksMessages::get_message_already_added(&ctx.data).await
                            }
                            WikiUrlStatus::Pending => {
                                AddLinksMessages::get_message_already_pending(&ctx.data).await
                            }
                            WikiUrlStatus::Removed => {
                                AddLinksMessages::get_message_previously_removed(&ctx.data).await
                            }
                        };
                        embed = embed.field(title, formatted, false);
                    }
                }

                let builder = CreateMessage::new()
                    .add_embed(embed)
                    .reference_message(MessageReference::from(&message))
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
                .exec(&FmbyDatabase::get_from_type_map(&ctx.data).await)
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
                    "Database error while handling AddLinksHandler::message"
                );
            }
        }
    }
}
