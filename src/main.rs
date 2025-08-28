use fmby_core::rss::RssConfig;
use poise::{
    Prefix,
    serenity_prelude::{self as serenity},
};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    let options = poise::FrameworkOptions {
        commands: fmby_commands::commands(),
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("||".into()),
            additional_prefixes: vec![Prefix::Literal("$")],
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
    let intents = serenity::GatewayIntents::all();

    let client = serenity::Client::builder(token, intents)
        .framework(framework)
        .event_handler(fmby_events::Handler)
        .data(Arc::new(fmby_core::structs::Data {
            time_started: std::time::Instant::now(),
            has_started: std::sync::atomic::AtomicBool::new(false),
            database: fmby_core::database::FmbyDatabase::init().await,
rss_config: RssConfig::default(),
        }))
        .await;

    client.unwrap().start().await.unwrap();
}
