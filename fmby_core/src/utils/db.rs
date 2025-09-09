use fmby_entities::{prelude::*, wiki_urls};
use poise::serenity_prelude as serenity;
use sea_orm::{Iterable, prelude::*};

#[serenity::async_trait]
pub trait WikiUrlFinder {
    async fn find_wiki_url_entries(
        &self,
        pool: &DatabaseConnection,
    ) -> Result<Vec<wiki_urls::Model>, DbErr>;
}

#[serenity::async_trait]
impl WikiUrlFinder for Vec<String> {
    async fn find_wiki_url_entries(
        &self,
        pool: &DatabaseConnection,
    ) -> Result<Vec<wiki_urls::Model>, DbErr> {
        if self.is_empty() {
            return Ok(Vec::new());
        }

        WikiUrls::find()
            .filter(wiki_urls::Column::Url.is_in(self))
            .all(pool)
            .await
    }
}

#[serenity::async_trait]
impl WikiUrlFinder for [String] {
    async fn find_wiki_url_entries(
        &self,
        pool: &DatabaseConnection,
    ) -> Result<Vec<wiki_urls::Model>, DbErr> {
        self.to_vec().find_wiki_url_entries(pool).await
    }
}

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
