mod rss_fetcher;
mod rss_manager;
mod rss_scheduler;
pub use rss_fetcher::*;
pub use rss_manager::*;
pub use rss_scheduler::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RssConfig {
    pub settings: RssSettings,
    pub fetcher: RssFetcherConfig,
    pub embed: RssEmbedConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RssSettings {
    pub default_check_interval: i32,
    pub max_entries_per_check: usize,
    pub max_concurrent_checks: usize,
    pub debug_force_post: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RssFetcherConfig {
    pub http_timeout_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RssEmbedConfig {
    pub color: u32,
    pub max_description_length: usize,
}

impl Default for RssConfig {
    fn default() -> Self {
        Self {
            settings: RssSettings {
                default_check_interval: 5,
                max_entries_per_check: 5,
                max_concurrent_checks: 5,
                debug_force_post: false,
            },
            fetcher: RssFetcherConfig {
                http_timeout_seconds: 30,
            },
            embed: RssEmbedConfig {
                color: 0x00D4AA,
                max_description_length: 300,
            },
        }
    }
}
