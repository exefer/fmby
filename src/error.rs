use poise::serenity_prelude::{Context, Permissions};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PermissionErrorType {
    #[error("Missing user permissions: {0}")]
    User(Permissions),
    #[error("Missing bot permissions: {0}")]
    Bot(Permissions),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Permissions(#[from] PermissionErrorType),
    #[error(transparent)]
    Database(#[from] sea_orm::DbErr),
    #[error(transparent)]
    FeedParse(#[from] feed_rs::parser::ParseFeedError),
    #[error(transparent)]
    Serenity(#[from] poise::serenity_prelude::Error),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Url(#[from] url::ParseError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Regex(#[from] regex::Error),
    #[error(transparent)]
    Image(#[from] image::ImageError),
}

#[expect(unused_variables, clippy::unused_async)]
pub async fn event_handler(ctx: &Context, error: Error) {}
