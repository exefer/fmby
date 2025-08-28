use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;

pub struct FmbyDatabase {
    pub pool: DatabaseConnection,
}

impl FmbyDatabase {
    pub async fn init() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set.");
        let mut conn_opts = ConnectOptions::new(database_url);

        conn_opts
            .min_connections(1)
            .max_connections(3)
            .connect_timeout(Duration::from_secs(8))
            .acquire_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8))
            .sqlx_logging(false);

        let pool = Database::connect(conn_opts)
            .await
            .expect("Failed to connect to database!");

        Self { pool }
    }
}
