use crate::server::execute::ExecuteToken;
use crate::{db, postoffice};
use anyhow::Result;
use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use kv_log_macro::{info, trace};
use sqlx::types::Uuid;
use sqlx::{Postgres, Transaction};
use std::collections::HashMap;
use std::fmt;

pub struct ProcessToken(pub Token);

// TODO - move this out into general code
#[derive(PartialEq, Hash, Eq, Clone, Debug)]
pub struct Token {
    pub task_id: Uuid,
    pub trigger_datetime: DateTime<Utc>,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<token {} @ {}>",
            self.task_id,
            self.trigger_datetime.to_rfc3339()
        )
    }
}

pub async fn process_tokens() -> Result<!> {
    let pool = db::get_pool();

    let token_rx = postoffice::receive_mail::<ProcessToken>().await?;
    let execute_tx = postoffice::post_mail::<ExecuteToken>().await?;

    info!("restoring tokens from database...");

    let mut tokens = HashMap::<Token, i32>::new();

    // first load all tokens from the DB
    let mut cursor = sqlx::query_as(
        "SELECT
            task.id,
            token.trigger_datetime,
            token.count,
            task.threshold
        FROM token
        JOIN task ON task.id = token.task_id
        WHERE token.count > 0",
    )
    .fetch(&pool);

    while let Some(row) = cursor.try_next().await? {
        let (task_id, trigger_datetime, count, threshold) = row;

        let token = Token {
            task_id,
            trigger_datetime,
        };

        trace!("restored token", {task_id: token.task_id.to_string(), count: count});

        if count >= threshold {
            execute_tx.send(ExecuteToken(token.clone())).await;
        }

        tokens.insert(token, count);
    }

    info!("done restoring tokens from database");

    loop {
        let token = token_rx.recv().await?.0;

        let count = tokens.entry(token.clone()).or_insert(0);
        *count += 1;

        let (threshold,) = sqlx::query_as::<_, (i32,)>(
            "SELECT threshold
            FROM task
            WHERE id = $1",
        )
        .bind(token.task_id)
        .fetch_one(&pool)
        .await?;

        trace!("count is {} (threshold {})", *count, threshold, {
            task_id: token.task_id.to_string(),
            trigger_datetime: token.trigger_datetime.to_rfc3339(),
        });
        if *count >= threshold {
            execute_tx.send(ExecuteToken(token)).await;
        }
    }
}

/// Adds a token to a task node
/// Usually you need to update some other state at the same time so it takes in a begun transaction
/// so you can do the other update before comitting (eg. update the trigger's last trigger time, or
/// update the state of the upstream task to be 'done')
/// After adding the token you have to send the token over to the process_tokens future to actually
/// check if the node has activated
pub async fn increment_token(txn: &mut Transaction<'_, Postgres>, token: &Token) -> Result<()> {
    trace!("incrementing token", {
        task_id: token.task_id.to_string(),
        trigger_datetime: token.trigger_datetime.to_rfc3339(),
    });

    sqlx::query(
        "INSERT INTO token(task_id, trigger_datetime, count, state)
            VALUES ($1, $2, 0, 'waiting')
            ON CONFLICT DO NOTHING",
    )
    .bind(token.task_id)
    .bind(token.trigger_datetime)
    .execute(&mut *txn)
    .await?;

    sqlx::query(
        "UPDATE token
            SET count = count + 1
            WHERE task_id = $1
            AND trigger_datetime = $2",
    )
    .bind(token.task_id)
    .bind(token.trigger_datetime)
    .execute(&mut *txn)
    .await?;

    Ok(())
}