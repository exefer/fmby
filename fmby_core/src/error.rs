#![allow(unused)]
use poise::serenity_prelude::{self as serenity, Permissions};
use std::fmt;

impl fmt::Display for PermissionErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PermissionErrorType::User(perms) => write!(f, "Missing user permissions: {perms}"),
            PermissionErrorType::Bot(perms) => write!(f, "Missing bot permissions: {perms}"),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Permissions(e) => write!(f, "{e}"),
            Error::Custom(e) => write!(f, "{e}"),
        }
    }
}

impl<E> From<E> for Error
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(e: E) -> Self {
        Error::Custom(Box::new(e))
    }
}

#[derive(Debug)]
pub enum PermissionErrorType {
    User(Permissions),
    Bot(Permissions),
}

#[derive(Debug)]
pub enum Error {
    Permissions(PermissionErrorType),
    Custom(Box<dyn std::error::Error + Send + Sync>),
}

#[expect(unused_variables, clippy::unused_async)]
pub async fn event_handler(ctx: &serenity::Context, error: Error) {}
