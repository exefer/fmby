use crate::{drama::DramaConfig, error::Error, rss::RssConfig};
pub type Context<'a> = poise::Context<'a, Data, Error>;
pub type Command = poise::Command<Data, Error>;

pub struct Data {
    pub time_started: std::time::Instant,
    pub has_started: std::sync::atomic::AtomicBool,
    pub database: crate::database::FmbyDatabase,
    pub rss_config: RssConfig,
    pub drama_config: DramaConfig,
}
