use sea_orm::entity::prelude::*;

use super::enums::RssFeedStatus;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "rss_feeds")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    #[sea_orm(column_type = "Text", unique_key = "uq_rss_feeds_url_channel_id")]
    pub url: String,
    #[sea_orm(column_type = "Text")]
    pub name: String,
    #[sea_orm(unique_key = "uq_rss_feeds_url_channel_id")]
    pub channel_id: i64,
    pub guild_id: i64,
    pub created_by: i64,
    pub created_at: DateTimeWithTimeZone,
    pub last_checked_at: DateTimeWithTimeZone,
    pub check_interval_minutes: i32,
    pub status: RssFeedStatus,
    #[sea_orm(has_many)]
    pub entries: HasMany<super::rss_feed_entries::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
