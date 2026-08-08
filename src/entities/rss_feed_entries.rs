use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "rss_feed_entries")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    #[sea_orm(unique_key = "uq_rss_feed_entries_feed_entry_id")]
    pub feed_id: Uuid,
    #[sea_orm(column_type = "Text", unique_key = "uq_rss_feed_entries_feed_entry_id")]
    pub entry_id: String,
    #[sea_orm(column_type = "Text")]
    pub title: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub link: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub thumbnail_url: Option<String>,
    pub published_at: Option<DateTimeWithTimeZone>,
    pub created_at: DateTimeWithTimeZone,
    pub message_id: Option<i64>,
    #[sea_orm(
        belongs_to,
        from = "feed_id",
        to = "id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    pub feed: BelongsTo<super::rss_feeds::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
