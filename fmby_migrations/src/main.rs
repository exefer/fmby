use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() {
    cli::run_cli(fmby_migrations::Migrator).await;
}
