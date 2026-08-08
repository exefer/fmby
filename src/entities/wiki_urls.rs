use sea_orm::entity::prelude::*;

use super::enums::WikiUrlStatus;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "wiki_urls")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32,
    #[sea_orm(column_type = "Text", unique_key = "uq_wiki_urls_url")]
    pub url: String,
    pub channel_id: Option<i64>,
    pub user_id: Option<i64>,
    pub message_id: Option<i64>,
    pub guild_id: Option<i64>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub status: WikiUrlStatus,
}

impl ActiveModelBehavior for ActiveModel {}
