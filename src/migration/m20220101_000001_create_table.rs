use async_trait::async_trait;
use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

use crate::entities::enums::{RssFeedStatus, RssFeedStatusEnum, WikiUrlStatus, WikiUrlStatusEnum};
use crate::entities::{prelude::*, rss_feed_entries, rss_feeds, wiki_urls};

#[derive(DeriveMigrationName)]
pub struct Migration;

const UQ_WIKI_URLS_URL: &str = "uq_wiki_urls_url";
const UQ_RSS_FEEDS_URL_CHANNEL_ID: &str = "uq_rss_feeds_url_channel_id";
const FK_RSS_FEED_ENTRIES_FEED_ID: &str = "fk_rss_feed_entries_feed_id";
const UQ_RSS_FEED_ENTRIES_FEED_ENTRY_ID: &str = "uq_rss_feed_entries_feed_entry_id";

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(WikiUrlStatusEnum)
                    .values(WikiUrlStatus::iden_values())
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(RssFeedStatusEnum)
                    .values(RssFeedStatus::iden_values())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(WikiUrls)
                    .if_not_exists()
                    .col(pk_auto(wiki_urls::Column::Id))
                    .col(text(wiki_urls::Column::Url))
                    .col(big_integer_null(wiki_urls::Column::ChannelId))
                    .col(big_integer_null(wiki_urls::Column::UserId))
                    .col(big_integer_null(wiki_urls::Column::MessageId))
                    .col(big_integer_null(wiki_urls::Column::GuildId))
                    .col(
                        timestamp_with_time_zone(wiki_urls::Column::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(wiki_urls::Column::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(custom(wiki_urls::Column::Status, WikiUrlStatusEnum))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(RssFeeds)
                    .if_not_exists()
                    .col(pk_uuid(rss_feeds::Column::Id))
                    .col(text(rss_feeds::Column::Url))
                    .col(text(rss_feeds::Column::Name))
                    .col(big_integer(rss_feeds::Column::ChannelId))
                    .col(big_integer(rss_feeds::Column::GuildId))
                    .col(big_integer(rss_feeds::Column::CreatedBy))
                    .col(
                        timestamp_with_time_zone(rss_feeds::Column::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(rss_feeds::Column::LastCheckedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(integer(rss_feeds::Column::CheckIntervalMinutes).default(5))
                    .col(custom(rss_feeds::Column::Status, RssFeedStatusEnum))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(RssFeedEntries)
                    .if_not_exists()
                    .col(pk_uuid(rss_feed_entries::Column::Id))
                    .col(uuid(rss_feed_entries::Column::FeedId))
                    .col(text(rss_feed_entries::Column::EntryId))
                    .col(text(rss_feed_entries::Column::Title))
                    .col(text_null(rss_feed_entries::Column::Link))
                    .col(text_null(rss_feed_entries::Column::Description))
                    .col(text_null(rss_feed_entries::Column::ThumbnailUrl))
                    .col(timestamp_with_time_zone_null(
                        rss_feed_entries::Column::PublishedAt,
                    ))
                    .col(
                        timestamp_with_time_zone(rss_feed_entries::Column::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(big_integer_null(rss_feed_entries::Column::MessageId))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(UQ_WIKI_URLS_URL)
                    .table(WikiUrls)
                    .col(wiki_urls::Column::Url)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(UQ_RSS_FEEDS_URL_CHANNEL_ID)
                    .table(RssFeeds)
                    .col(rss_feeds::Column::Url)
                    .col(rss_feeds::Column::ChannelId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(UQ_RSS_FEED_ENTRIES_FEED_ENTRY_ID)
                    .table(RssFeedEntries)
                    .col(rss_feed_entries::Column::FeedId)
                    .col(rss_feed_entries::Column::EntryId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_RSS_FEED_ENTRIES_FEED_ID)
                    .from(RssFeedEntries, rss_feed_entries::Column::FeedId)
                    .to(RssFeeds, rss_feeds::Column::Id)
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
                    .table(RssFeedEntries)
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
                    .name(UQ_RSS_FEED_ENTRIES_FEED_ENTRY_ID)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(RssFeedEntries).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(RssFeeds).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(WikiUrls).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(RssFeedStatusEnum).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(WikiUrlStatusEnum).to_owned())
            .await?;

        Ok(())
    }
}
