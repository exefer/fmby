use fmby_entities::{prelude::*, rss_feed_entries, rss_feeds, sea_orm_active_enums::RssFeedStatus};
use sea_orm::{Condition, QueryOrder, prelude::*, sea_query::OnConflict};

pub struct RssManager {
    pool: DatabaseConnection,
}

impl RssManager {
    pub fn new(pool: DatabaseConnection) -> Self {
        Self { pool }
    }

    pub async fn add_feed(
        &self,
        new_feed: rss_feeds::ActiveModel,
    ) -> Result<rss_feeds::Model, DbErr> {
        let feed = RssFeeds::insert(new_feed)
            .exec_with_returning(&self.pool)
            .await?;

        Ok(feed)
    }

    pub async fn remove_feed(&self, id: Uuid) -> Result<bool, DbErr> {
        let result = RssFeeds::delete_by_id(id).exec(&self.pool).await?;

        Ok(result.rows_affected > 0)
    }

    pub async fn list_feeds(&self, guild_id: u64) -> Result<Vec<rss_feeds::Model>, DbErr> {
        let feeds = RssFeeds::find()
            .filter(rss_feeds::Column::GuildId.eq(guild_id))
            .all(&self.pool)
            .await?;

        Ok(feeds)
    }

    pub async fn get_feed(&self, id: Uuid) -> Result<Option<rss_feeds::Model>, DbErr> {
        let feed = RssFeeds::find_by_id(id).one(&self.pool).await?;

        Ok(feed)
    }

    pub async fn get_feeds_to_check(&self) -> Result<Vec<rss_feeds::Model>, DbErr> {
        let feeds = RssFeeds::find()
            .filter(
                Condition::all()
                    .add(rss_feeds::Column::Status.eq(RssFeedStatus::Active))
                    .add(
                        Expr::col(rss_feeds::Column::LastCheckedAt).lt(Expr::current_timestamp()
                            .sub(
                                Expr::col(rss_feeds::Column::CheckIntervalMinutes)
                                    .mul(Expr::cust("INTERVAL '1 minute'")),
                            )),
                    ),
            )
            .order_by_asc(rss_feeds::Column::LastCheckedAt)
            .all(&self.pool)
            .await?;

        Ok(feeds)
    }

    pub async fn update_last_checked_at(&self, id: Uuid) -> Result<(), DbErr> {
        RssFeeds::update_many()
            .col_expr(
                rss_feeds::Column::LastCheckedAt,
                Expr::current_timestamp().into(),
            )
            .filter(rss_feeds::Column::Id.eq(id))
            .exec(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_entry_message_id(
        &self,
        entry_id: Uuid,
        message_id: u64,
    ) -> Result<(), DbErr> {
        RssFeedEntries::update_many()
            .col_expr(rss_feed_entries::Column::MessageId, Expr::value(message_id))
            .filter(rss_feed_entries::Column::Id.eq(entry_id))
            .exec(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn insert_feed_entries(
        &self,
        entries: Vec<rss_feed_entries::ActiveModel>,
    ) -> Result<Vec<rss_feed_entries::Model>, DbErr> {
        let feed_entries = RssFeedEntries::insert_many(entries)
            .on_conflict(
                OnConflict::columns([
                    rss_feed_entries::Column::FeedId,
                    rss_feed_entries::Column::EntryId,
                ])
                .do_nothing()
                .to_owned(),
            )
            .exec_with_returning_many(&self.pool)
            .await?;

        Ok(feed_entries)
    }
}
