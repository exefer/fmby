use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

const UQ_WIKI_URLS_URL: &str = "uq_wiki_urls_url";
const UQ_RSS_FEEDS_URL_CHANNEL_ID: &str = "uq_rss_feeds_url_channel_id";
const IDX_RSS_FEEDS_ACTIVE_LAST_CHECKED_AT: &str = "idx_rss_feeds_active_last_checked_at";
const FK_RSS_FEED_ENTRIES_FEED_ID: &str = "fk_rss_feed_entries_feed_id";
const UQ_RSS_FEED_ENTRIES_FEED_ENTRY_ID: &str = "uq_rss_feed_entries_feed_entry_id";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(WikiUrlStatus::Enum)
                    .values([
                        WikiUrlStatus::Added,
                        WikiUrlStatus::Removed,
                        WikiUrlStatus::Pending,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(RssFeedStatus::Enum)
                    .values([RssFeedStatus::Active, RssFeedStatus::Inactive])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(WikiUrls::Table)
                    .if_not_exists()
                    .col(pk_auto(WikiUrls::Id))
                    .col(text(WikiUrls::Url))
                    .col(big_integer_null(WikiUrls::ChannelId))
                    .col(big_integer_null(WikiUrls::UserId))
                    .col(big_integer_null(WikiUrls::MessageId))
                    .col(big_integer_null(WikiUrls::GuildId))
                    .col(
                        timestamp_with_time_zone(WikiUrls::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(WikiUrls::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(custom(WikiUrls::Status, WikiUrlStatus::Enum))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(RssFeeds::Table)
                    .if_not_exists()
                    .col(pk_uuid(RssFeeds::Id))
                    .col(text(RssFeeds::Url))
                    .col(text(RssFeeds::Name))
                    .col(big_integer(RssFeeds::ChannelId))
                    .col(big_integer(RssFeeds::GuildId))
                    .col(big_integer(RssFeeds::CreatedBy))
                    .col(
                        timestamp_with_time_zone(RssFeeds::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(RssFeeds::LastCheckedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(integer(RssFeeds::CheckIntervalMinutes).default(5))
                    .col(custom(RssFeeds::Status, RssFeedStatus::Enum))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(RssFeedEntries::Table)
                    .if_not_exists()
                    .col(pk_uuid(RssFeedEntries::Id))
                    .col(uuid(RssFeedEntries::FeedId))
                    .col(text(RssFeedEntries::EntryId))
                    .col(text(RssFeedEntries::Title))
                    .col(text_null(RssFeedEntries::Link))
                    .col(text_null(RssFeedEntries::Description))
                    .col(text_null(RssFeedEntries::ThumbnailUrl))
                    .col(timestamp_with_time_zone_null(RssFeedEntries::PublishedAt))
                    .col(
                        timestamp_with_time_zone(RssFeedEntries::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(big_integer_null(RssFeedEntries::MessageId))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(UQ_WIKI_URLS_URL)
                    .table(WikiUrls::Table)
                    .col(WikiUrls::Url)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(UQ_RSS_FEEDS_URL_CHANNEL_ID)
                    .table(RssFeeds::Table)
                    .col(RssFeeds::Url)
                    .col(RssFeeds::ChannelId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(IDX_RSS_FEEDS_ACTIVE_LAST_CHECKED_AT)
                    .table(RssFeeds::Table)
                    .col(RssFeeds::Status)
                    .col(RssFeeds::LastCheckedAt)
                    .and_where(Expr::col(RssFeeds::Status).eq(RssFeedStatus::Active.to_string()))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(UQ_RSS_FEED_ENTRIES_FEED_ENTRY_ID)
                    .table(RssFeedEntries::Table)
                    .col(RssFeedEntries::FeedId)
                    .col(RssFeedEntries::EntryId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_RSS_FEED_ENTRIES_FEED_ID)
                    .from(RssFeedEntries::Table, RssFeedEntries::FeedId)
                    .to(RssFeeds::Table, RssFeeds::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(RssFeedEntries::Table)
                    .name(FK_RSS_FEED_ENTRIES_FEED_ID)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(Index::drop().name(UQ_WIKI_URLS_URL).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name(UQ_RSS_FEEDS_URL_CHANNEL_ID).to_owned())
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name(IDX_RSS_FEEDS_ACTIVE_LAST_CHECKED_AT)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name(UQ_RSS_FEED_ENTRIES_FEED_ENTRY_ID)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(RssFeedEntries::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(RssFeeds::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(WikiUrls::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(RssFeedStatus::Enum).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(WikiUrlStatus::Enum).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum WikiUrls {
    Table,
    Id,
    Url,
    ChannelId,
    UserId,
    MessageId,
    GuildId,
    CreatedAt,
    UpdatedAt,
    Status,
}

#[derive(DeriveIden)]
pub enum WikiUrlStatus {
    #[sea_orm(iden = "wiki_url_status")]
    Enum,
    #[sea_orm(iden = "added")]
    Added,
    #[sea_orm(iden = "removed")]
    Removed,
    #[sea_orm(iden = "pending")]
    Pending,
}

#[derive(DeriveIden)]
pub enum RssFeeds {
    Table,
    Id,
    Url,
    Name,
    ChannelId,
    GuildId,
    CreatedBy,
    CreatedAt,
    LastCheckedAt,
    CheckIntervalMinutes,
    Status,
}

#[derive(DeriveIden)]
pub enum RssFeedStatus {
    #[sea_orm(iden = "rss_feed_status")]
    Enum,
    #[sea_orm(iden = "active")]
    Active,
    #[sea_orm(iden = "inactive")]
    Inactive,
}

#[derive(DeriveIden)]
pub enum RssFeedEntries {
    Table,
    Id,
    FeedId,
    EntryId,
    Title,
    Link,
    Description,
    ThumbnailUrl,
    PublishedAt,
    CreatedAt,
    MessageId,
}
