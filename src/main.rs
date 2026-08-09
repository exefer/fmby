mod background_task;
mod channels;
mod commands;
mod constants;
mod db;
mod drama;
mod entities;
mod error;
mod events;
mod formatters;
mod message;
mod migration;
mod rss;
mod stale_remover;
mod types;
mod url;
mod wiki;

use std::env;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use migration::Migrator;
use poise::serenity_prelude::{self as serenity, GatewayIntents};
use sea_orm::{ConnectOptions, Database};
use sea_orm_migration::MigratorTraitSelf;

#[cfg(unix)]
async fn shutdown_signal() {
    use tokio::signal::unix::{SignalKind, signal};

    let (mut s1, mut s2, mut s3) = (
        signal(SignalKind::hangup()).unwrap(),
        signal(SignalKind::interrupt()).unwrap(),
        signal(SignalKind::terminate()).unwrap(),
    );

    tokio::select! {
        _ = s1.recv() => {},
        _ = s2.recv() => {},
        _ = s3.recv() => {},
    }
}

#[cfg(windows)]
async fn shutdown_signal() {
    use tokio::signal::windows;

    let (mut s1, mut s2) = (windows::ctrl_c().unwrap(), windows::ctrl_break().unwrap());

    tokio::select! {
        _ = s1.recv() => {},
        _ = s2.recv() => {},
    }
}

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt().init();

    let _ = rustls::crypto::ring::default_provider().install_default();

    let options = poise::FrameworkOptions {
        commands: commands::commands(),
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

    let token = serenity::Token::from_env("BOT_TOKEN").expect("BOT_TOKEN is not set");
    let intents = GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::DIRECT_MESSAGE_REACTIONS;

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set.");
    let mut conn_opts = ConnectOptions::new(database_url);
    conn_opts
        .min_connections(1)
        .max_connections(5)
        .sqlx_logging(false);
    let pool = Database::connect(conn_opts)
        .await
        .expect("Failed to connect to database!");

    Migrator
        .up(&pool, None)
        .await
        .expect("Failed to run database migrations!");

    let mut client = serenity::Client::builder(token, intents)
        .framework(Box::new(framework))
        .event_handler(Arc::new(events::Handler))
        .data(Arc::new(types::Data {
            time_started: Instant::now(),
            has_started: AtomicBool::new(false),
            pool,
            rss_config: rss::RssConfig::default(),
            drama_config: drama::DramaConfig::from_config(),
        }))
        .await
        .expect("failed to create client");

    let shutdown = client.shard_manager.get_shutdown_trigger();

    tokio::spawn(async {
        shutdown_signal().await;
        shutdown();
    });

    client.start().await.unwrap();
}
