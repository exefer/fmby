use fmby_core::constants::{AUTO_THREAD_MAPPINGS, FmhyChannel};
use fmby_core::{
    structs::Data,
    utils::{db::WikiUrlFinder, formatters::UrlFormatter, url::extract_urls},
};
use fmby_entities::{prelude::*, sea_orm_active_enums::WikiUrlStatus, wiki_urls};
use poise::serenity_prelude::*;
use sea_orm::{ActiveValue::*, Iterable, prelude::*, sea_query::OnConflict};

fn is_add_links_channel(id: u64) -> bool {
    id == FmhyChannel::AddLinks.id() || id == FmhyChannel::NsfwAddLinks.id()
}

pub async fn on_message(ctx: &Context, message: &Message) {
    for (channel_id, needle) in AUTO_THREAD_MAPPINGS.iter() {
        if message.channel_id != *channel_id {
            continue;
        }

        if needle.is_none_or(|n| message.content.contains(n)) {
            let channel_id = message.channel_id.expect_channel();
            let _ = channel_id
                .create_thread_from_message(
                    &ctx.http,
                    message.id,
                    CreateThread::new("Please keep discussions in here!")
                        .audit_log_reason("Auto thread created by FMBY bot"),
                )
                .await;
            return;
        }
    }

    let Some(ref urls) = extract_urls(&message.content) else {
        return;
    };

    let wiki_entries = urls
        .find_wiki_url_entries(&ctx.data::<Data>().database.pool)
        .await;

    match wiki_entries {
        Ok(wiki_entries) if !wiki_entries.is_empty() => {
            let mut embed = CreateEmbed::new().title("Warning").color(Color::ORANGE);

            for status in WikiUrlStatus::iter() {
                if let Some(formatted) = wiki_entries.format_for_embed(&status) {
                    let title = match status {
                        WikiUrlStatus::Added => "Link(s) already in the wiki:",
                        WikiUrlStatus::Pending => "Links(s) already in queue:",
                        WikiUrlStatus::Removed => "Links(s) previously removed from the wiki:",
                    };
                    embed = embed.field(title, formatted, false);
                }
            }

            let builder = CreateMessage::new()
                .add_embed(embed)
                .reference_message(MessageReference::from(message))
                .allowed_mentions(CreateAllowedMentions::new().replied_user(true));

            let _ = message.channel_id.send_message(&ctx.http, builder).await;
        }
        Ok(_) => {
            if !is_add_links_channel(message.channel_id.get()) {
                return;
            }

            let _ = WikiUrls::insert_many(
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
            .await;
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                guild_id = ?message.guild_id.map(|g| g.get()),
                channel_id = %message.channel_id.get(),
                message_id = %message.id.get(),
                user_id = %message.author.id.get(),
                urls = ?urls,
                "Failed to insert wiki URLs into the database"
            );
        }
    }
}
