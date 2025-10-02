use fmby_entities::{prelude::*, sea_orm_active_enums::WikiUrlStatus, wiki_urls};
use poise::serenity_prelude::Message;
use sea_orm::{ActiveValue::*, IntoActiveModel, Iterable, prelude::*, sqlx::types::chrono::Utc};

use crate::constants::FmhyChannel;

pub trait ChunkSize {
    fn chunk_size() -> usize;
}

impl<E> ChunkSize for E
where
    E: EntityTrait,
{
    fn chunk_size() -> usize {
        let num_columns = E::Column::iter().count() as u16;
        if num_columns == 0 {
            0
        } else {
            (u16::MAX / num_columns) as usize
        }
    }
}

pub fn infer_wiki_url_status(channel_id: u64) -> Option<WikiUrlStatus> {
    match channel_id {
        FmhyChannel::ADD_LINKS | FmhyChannel::NSFW_ADD_LINKS => Some(WikiUrlStatus::Pending),
        FmhyChannel::RECENTLY_ADDED | FmhyChannel::NSFW_RECENTLY_ADDED => {
            Some(WikiUrlStatus::Added)
        }
        FmhyChannel::DEAD_SITES | FmhyChannel::REMOVE_SITES | FmhyChannel::NSFW_REMOVED => {
            Some(WikiUrlStatus::Removed)
        }
        _ => None,
    }
}

pub async fn get_wiki_urls_by_urls(
    urls: &[String],
    pool: &DatabaseConnection,
) -> Option<Vec<wiki_urls::Model>> {
    if urls.is_empty() {
        return None;
    }

    WikiUrls::find()
        .filter(wiki_urls::Column::Url.is_in(urls))
        .all(pool)
        .await
        .ok()
}

pub async fn update_wiki_urls_with_message(
    entries: Vec<wiki_urls::Model>,
    message: &Message,
    status: WikiUrlStatus,
    pool: &DatabaseConnection,
) {
    for mut entry in entries.into_iter().map(IntoActiveModel::into_active_model) {
        entry.user_id = Set(Some(message.author.id.get() as i64));
        entry.message_id = Set(Some(message.id.get() as i64));
        entry.channel_id = Set(Some(message.channel_id.get() as i64));
        entry.updated_at = Set(Utc::now().into());
        entry.status = Set(status);

        let _ = entry.update(pool).await;
    }
}
