use std::sync::atomic::AtomicBool;
use std::time::Instant;

use sea_orm::DatabaseConnection;

use crate::drama::DramaConfig;
use crate::error::Error;
use crate::rss::RssConfig;

pub type Context<'a> = poise::Context<'a, Data, Error>;
pub type Command = poise::Command<Data, Error>;

pub struct Data {
    pub time_started: Instant,
    pub has_started: AtomicBool,
    pub pool: DatabaseConnection,
    pub rss_config: RssConfig,
    pub drama_config: DramaConfig,
}
