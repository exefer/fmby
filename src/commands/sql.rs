use poise::CreateReply;
use poise::serenity_prelude::MessageFlags;
use sea_orm::{ConnectionTrait, Statement};

use super::{Command, Context, Error};

/// Executes a raw SQL command and replies with the number of affected rows or an error
#[poise::command(prefix_command, owners_only)]
async fn sql_exec(ctx: Context<'_>, sql: String) -> Result<(), Error> {
    match ctx.data().pool.execute_unprepared(&sql).await {
        Ok(result) => {
            ctx.reply(format!("Rows affected: {}", result.rows_affected()))
                .await?;
        }
        Err(e) => {
            ctx.reply(e.to_string()).await?;
        }
    }

    Ok(())
}

/// Executes a SQL query, optionally pretty-prints results, and replies or returns an error
#[poise::command(prefix_command, owners_only)]
async fn sql_query(ctx: Context<'_>, sql: String, #[flag] pretty: bool) -> Result<(), Error> {
    match ctx
        .data()
        .pool
        .query_all_raw(Statement::from_string(
            ctx.data().pool.get_database_backend(),
            sql,
        ))
        .await
    {
        Ok(result) => {
            let rows: Vec<_> = result.iter().filter_map(|q| q.try_as_pg_row()).collect();

            let formatted = if pretty {
                format!("{rows:#?}")
            } else {
                format!("{rows:?}")
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
            ctx.reply(e.to_string()).await?;
        }
    }

    Ok(())
}

pub fn commands() -> [Command; 2] {
    [sql_exec(), sql_query()]
}
