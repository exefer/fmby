use crate::{Context, Error};
use poise::{CreateReply, serenity_prelude::MessageFlags};
use sea_orm::{ConnectionTrait, Statement};

/// Executes a raw SQL command and replies with the number of affected rows or an error
#[poise::command(prefix_command, owners_only)]
pub async fn sql_exec(ctx: Context<'_>, sql: String) -> Result<(), Error> {
    match ctx.data().database.pool.execute_unprepared(&sql).await {
        Ok(result) => {
            ctx.reply(format!("Rows affected: {}", result.rows_affected()))
                .await?;
        }
        Err(e) => {
            ctx.reply(format!("{}", e)).await?;
        }
    };

    Ok(())
}

/// Executes a SQL query, optionally pretty-prints results, and replies or returns an error
#[poise::command(prefix_command, owners_only)]
pub async fn sql_query(ctx: Context<'_>, sql: String, pretty: Option<bool>) -> Result<(), Error> {
    match ctx
        .data()
        .database
        .pool
        .query_all(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            sql,
        ))
        .await
    {
        Ok(result) => {
            let rows: Vec<_> = result.iter().filter_map(|q| q.try_as_pg_row()).collect();

            let formatted = if pretty.unwrap_or(false) {
                format!("{:#?}", rows)
            } else {
                format!("{:?}", rows)
            };

            ctx.send(
                CreateReply::new()
                    .content(formatted)
                    .reply(true)
                    .flags(MessageFlags::SUPPRESS_EMBEDS),
            )
            .await?;
        }
        Err(e) => {
            ctx.reply(format!("{}", e)).await?;
        }
    }

    Ok(())
}

#[must_use]
pub fn commands() -> [crate::Command; 2] {
    [sql_exec(), sql_query()]
}
