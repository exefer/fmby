mod config;
mod constants;
mod error;
mod handlers;
mod macros;
mod shared;
mod utils;
use crate::{
    config::MessagesConfig,
    handlers::{AddLinksHandler, BookmarkHandler, LinkTestingHandler},
    shared::FmbyDatabase,
};
use poise::serenity_prelude as serenity;
use sea_orm::{ConnectOptions, Database};
use std::{env, sync::Arc, time::Duration};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let token = env::var("BOT_TOKEN").expect("Expected a token in the environment");
    let db_url =
        env::var("DATABASE_URL").expect("Expected a database connection url in the environment");
    let mut conn_opts = ConnectOptions::new(db_url);

    conn_opts
        .max_connections(20)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true);

    let conn = Database::connect(conn_opts)
        .await
        .expect("Database connection failed");

    let intents = serenity::GatewayIntents::all();
    let options = poise::FrameworkOptions {
        commands: vec![],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("~".into()),
            edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                Duration::from_secs(3600),
            ))),
            additional_prefixes: vec![
                poise::Prefix::Literal("hey bot,"),
                poise::Prefix::Literal("hey bot"),
            ],
            ..Default::default()
        },
        // This code is run before every command
        pre_command: |ctx: Context| {
            Box::pin(async move {
                println!("Executing command {}...", ctx.command().qualified_name);
            })
        },
        // This code is run after a command if it was successful (returned Ok)
        post_command: |ctx: Context| {
            Box::pin(async move {
                println!("Executed command {}!", ctx.command().qualified_name);
            })
        },
        // Every command invocation must pass this check to continue execution
        command_check: Some(|ctx: Context| Box::pin(async move { Ok(true) })),
        // Enforce command checks even for owners (enforced by default)
        // Set to true to bypass checks, which is useful for testing
        skip_checks_for_owners: false,
        event_handler: |_ctx, event, _framework, _data| {
            Box::pin(async move {
                println!(
                    "Got an event in event handler: {:?}",
                    event.snake_case_name()
                );

                Ok(())
            })
        },
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                println!("Logged in as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .options(options)
        .build();

    let mut client = serenity::Client::builder(&token, intents)
        .event_handler(AddLinksHandler)
        .event_handler(BookmarkHandler)
        .event_handler(LinkTestingHandler)
        .framework(framework)
        .await
        .expect("Error creating client");
    {
        let mut data = client.data.write().await;
        data.insert::<FmbyDatabase>(Arc::new(tokio::sync::RwLock::new(conn)));
        data.insert::<MessagesConfig>(Arc::new(tokio::sync::RwLock::new(
            config::MessagesConfig::default(),
        )));
    }

    if let Err(err) = client.start_shards(2).await {
        println!("Client error: {}", err);
    }
}
