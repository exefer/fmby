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
use sea_orm::{ConnectOptions, Database};
use serenity::prelude::*;
use std::{env, sync::Arc, time::Duration};

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

    let intents = GatewayIntents::all();

    let mut client = Client::builder(&token, intents)
        .event_handler(AddLinksHandler)
        .event_handler(BookmarkHandler)
        .event_handler(LinkTestingHandler)
        .await
        .expect("Error creating client");
    {
        let mut data = client.data.write().await;
        data.insert::<FmbyDatabase>(Arc::new(RwLock::new(conn)));
        data.insert::<MessagesConfig>(Arc::new(RwLock::new(config::MessagesConfig::default())));
    }

    if let Err(err) = client.start_shards(2).await {
        println!("Client error: {}", err);
    }
}
