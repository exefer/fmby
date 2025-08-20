use crate::config;
use sea_orm::DatabaseConnection;
use serenity::prelude::{TypeMap, TypeMapKey};
use std::{ops::Deref, sync::Arc};
use tokio::sync::RwLock;

pub async fn get<K>(data: &Arc<RwLock<TypeMap>>) -> K::Value
where
    K: TypeMapKey + Send + Sync + 'static,
    K::Value: Send + Sync + 'static + Clone,
{
    let map = data.read().await;
    map.get::<K>().expect("TypeMap key missing").clone()
}

impl TypeMapKey for config::MessagesConfig {
    type Value = Arc<RwLock<config::MessagesConfig>>;
}

pub struct FmbyDatabase(DatabaseConnection);

impl Deref for FmbyDatabase {
    type Target = DatabaseConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FmbyDatabase {
    /// Get a cloned [`DatabaseConnection`] from the [`Typemap`].
    ///
    /// # Example
    /// ```ignore
    /// use crate::shared::FmbyDatabase;
    ///
    /// async fn run_example(ctx: &poise::Context<'_, (), ()>) -> anyhow::Result<()> {
    ///     let conn = FmbyDatabase::get_from_type_map(&ctx.data).await;
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_from_type_map(data: &Arc<RwLock<TypeMap>>) -> DatabaseConnection {
        let conn_lock = crate::shared::get::<FmbyDatabase>(data).await;
        let conn = conn_lock.read().await;
        conn.clone()
    }
    /// Run a closure with a cloned [`DatabaseConnection`] from the [`TypeMap`].
    ///
    /// # Example
    /// ```ignore
    /// use crate::shared::FmbyDatabase;
    ///
    /// async fn run_example(ctx: &poise::Context<'_, (), ()>) -> anyhow::Result<()> {
    ///     FmbyDatabase::with_connection(&ctx.data, |conn| async {
    ///         /// Code...
    ///     }).await;
    ///     Ok(())
    /// }
    /// ```
    pub async fn with_connection<F, Fut, R>(data: &Arc<RwLock<TypeMap>>, f: F) -> R
    where
        F: FnOnce(DatabaseConnection) -> Fut,
        Fut: Future<Output = R>,
    {
        let conn = Self::get_from_type_map(data).await;
        f(conn).await
    }
}

impl TypeMapKey for FmbyDatabase {
    type Value = Arc<RwLock<DatabaseConnection>>;
}
