use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "rss_feed_status",
    rename_all = "snake_case"
)]
pub enum RssFeedStatus {
    Active,
    Inactive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "wiki_url_status",
    rename_all = "snake_case"
)]
pub enum WikiUrlStatus {
    Added,
    Removed,
    Pending,
}
