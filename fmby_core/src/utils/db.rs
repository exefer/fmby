use fmby_entities::{prelude::*, wiki_urls};
use poise::serenity_prelude as serenity;
use sea_orm::{Iterable, prelude::*};

#[serenity::async_trait]
pub trait WikiUrlFinder {
    /// Look up multiple wiki URL entries in a single database query.
    ///
    /// ## Arguments
    ///
    /// * `pool` - Reference to an active [`DatabaseConnection`].
    /// * `urls` - Slice of URLs to look up.
    ///
    /// ## Returns
    ///
    /// A `Result` containing:
    /// - `Ok(Vec<wiki_urls::Model>)`: all database rows where `wiki_urls.url`
    ///   matches one of the given `urls`. The vector may be empty if none of the
    ///   provided URLs exist in the database.
    /// - `Err(DbErr)`: if the query fails.
    ///
    /// ### Notes
    ///
    /// - This function performs one `SELECT ... WHERE url IN (...)` query instead
    ///   of executing a separate query per URL.
    /// - The returned vector does **not** preserve the order of the input URLs.
    ///   It will be in the order returned by the database.
    /// - If you need to know which input URLs were missing, compare the returned
    ///   models’ `url` field against your original list.
    ///
    /// ## Example
    ///
    /// ```ignore
    /// let urls = vec![
    ///     "example.com".to_string(),
    ///     "example.org".to_string(),
    /// ];
    ///
    /// let entries = urls.find_wiki_url_entries(&pool).await?;
    ///
    /// for entry in entries {
    ///     println!("Found: {} -> {:?}", entry.url, entry);
    /// }
    /// ```
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
