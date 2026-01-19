use poise::serenity_prelude::{self as serenity, GatewayIntents};
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"))
        .add_directive("wordcloud=off".parse().unwrap());

    tracing_subscriber::fmt().with_env_filter(filter).init();

    let options = poise::FrameworkOptions {
        commands: fmby_commands::commands(),
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some(r"\".into()),
            mention_as_prefix: true,
            execute_untracked_edits: false,
            case_insensitive_commands: true,
            edit_tracker: None,
            ..Default::default()
        },
        skip_checks_for_owners: false,
        ..Default::default()
    };

    let framework = poise::Framework::new(options);

    let token = serenity::Token::from_env("FMBY_TOKEN").expect("FMBY_TOKEN is not set.");
    let intents = GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::DIRECT_MESSAGE_REACTIONS;

    let client = serenity::Client::builder(token, intents)
        .framework(Box::new(framework))
        .event_handler(Arc::new(fmby_events::Handler))
        .data(Arc::new(fmby_core::structs::Data {
            time_started: std::time::Instant::now(),
            has_started: std::sync::atomic::AtomicBool::new(false),
            database: fmby_core::database::FmbyDatabase::init().await,
            rss_config: fmby_core::rss::RssConfig::default(),
            drama_config: fmby_core::drama::DramaConfig::from_config(),
        }))
        .await;

    client.unwrap().start().await.unwrap();
}
