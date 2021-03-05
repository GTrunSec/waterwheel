use crate::postoffice;
use crate::util::spawn_retry;
use crate::{amqp, db};
use anyhow::Result;

mod api;
mod execute;
mod progress;
pub mod stash;
pub mod status;
pub mod tokens;
mod trigger_time;
mod triggers;

pub async fn run_server() -> Result<()> {
    postoffice::open()?;

    stash::load_keys()?;

    db::create_pool().await?;
    amqp::amqp_connect().await?;

    spawn_retry("triggers", triggers::process_triggers);
    spawn_retry("tokens", tokens::process_tokens);
    spawn_retry("executions", execute::process_executions);
    spawn_retry("progress", progress::process_progress);

    api::serve().await?;

    Ok(())
}
