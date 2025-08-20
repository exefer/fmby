use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub enum WikiUrls {
    Table,
    Id,
    Url,
    Name,
    ChannelId,
    UserId,
    MessageId,
    GuildId,
    Status,
}

/// Represents the current status of a wiki URL in the database.
///
/// This enum is stored in PostgreSQL as a custom enum type [`WikiUrlStatus`]
/// and indicates whether a URL has been accepted, rejected, or is awaiting review.
#[derive(DeriveIden)]
pub enum WikiUrlStatus {
    #[sea_orm(iden = "wiki_url_status")]
    Enum,
    #[sea_orm(iden = "added")]
    Added,
    #[sea_orm(iden = "rejected")]
    Removed,
    #[sea_orm(iden = "pending")]
    Pending,
}
