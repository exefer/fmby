use crate::enums::{WikiUrlStatus, WikiUrls};
use sea_orm_migration::prelude::{extension::postgres::Type, *};

#[derive(DeriveMigrationName)]
pub struct Migration;

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
            .create_table(
                Table::create()
                    .table(WikiUrls::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(WikiUrls::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(WikiUrls::Url)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(WikiUrls::Name).string().null())
                    .col(ColumnDef::new(WikiUrls::ChannelId).big_integer().null())
                    .col(ColumnDef::new(WikiUrls::UserId).big_integer().null())
                    .col(ColumnDef::new(WikiUrls::MessageId).big_integer().null())
                    .col(ColumnDef::new(WikiUrls::GuildId).big_integer().null())
                    .col(
                        ColumnDef::new(WikiUrls::Status)
                            .custom(WikiUrlStatus::Enum)
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(WikiUrls::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(WikiUrlStatus::Enum).to_owned())
            .await?;

        Ok(())
    }
}
