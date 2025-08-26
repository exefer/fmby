use crate::{Context, Error};
use sea_orm::{ConnectionTrait, Statement};

#[poise::command(prefix_command, owners_only)]
pub async fn sql_exec(ctx: Context<'_>, sql: String) -> Result<(), Error> {
    match ctx.data().database.pool.execute_unprepared(&sql).await {
        Ok(res) => {
            ctx.reply(format!("Rows affected: {}", res.rows_affected()))
                .await?;
        }
        Err(err) => {
            ctx.reply(format!("{}", err)).await?;
        }
    };

    Ok(())
}

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
        Ok(exec) => {
            use sea_orm::sqlx::postgres::PgRow;
            let rows: Vec<&PgRow> = exec.iter().filter_map(|res| res.try_as_pg_row()).collect();

            let formatted = if pretty.unwrap_or(false) {
                format!("{:#?}", rows)
            } else {
                format!("{:?}", rows)
            };

            ctx.reply(formatted).await?;
        }
        Err(err) => {
            ctx.reply(format!("{}", err)).await?;
        }
    }

    Ok(())
}

pub fn commands() -> [crate::Command; 2] {
    [sql_exec(), sql_query()]
}
