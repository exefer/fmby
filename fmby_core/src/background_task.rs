use std::time::Duration;

use poise::serenity_prelude as serenity;
use tokio::time::MissedTickBehavior;

use crate::error::Error;

/// Trait for a background task that can be run periodically on Tokio.
#[serenity::async_trait]
pub trait BackgroundTask: Sized + Send + 'static {
    /// Create a new instance of the task using the provided `Context`.
    /// This is called once before the task starts running.
    async fn init(ctx: serenity::Context) -> Result<Self, Error>;

    /// How often the task should be run.
    /// This gets called after every call to `run()`.
    fn interval(&mut self) -> Duration;

    /// Run the background task.
    ///
    /// This gets called every `interval()`.
    async fn run(&mut self);

    /// Timeout for the task.
    ///
    /// If this returns `None`, the task will never time out.
    /// This gets called just before every call to `run()`.
    /// If the task times out, its future will be dropped, and after `interval()` has passed, it will be rerun.
    fn timeout(&mut self) -> Option<Duration> {
        None
    }
}

/// Starts a background task that implements [`BackgroundTask`] on Tokio.
pub async fn start_background_task<T>(ctx: &serenity::Context)
where
    T: BackgroundTask,
{
    let mut task = match T::init(ctx.clone()).await {
        Ok(task) => task,
        Err(e) => {
            tracing::error!(
                "Failed to init background task {}: {}",
                std::any::type_name::<T>(),
                e
            );
            return;
        }
    };

    let mut ticker = tokio::time::interval(task.interval());
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    tokio::spawn(async move {
        loop {
            ticker.tick().await;

            if let Some(timeout) = task.timeout() {
                if tokio::time::timeout(timeout, task.run()).await.is_err() {
                    tracing::warn!("Background task {} timed out", std::any::type_name::<T>());
                }
            } else {
                task.run().await;
            }
        }
    });

    tracing::info!("Started background task {}", std::any::type_name::<T>());
}
