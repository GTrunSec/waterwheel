use crate::amqp::get_amqp_channel;
use crate::messages::{TaskDef, TaskResult, WorkerHeartbeat};
use crate::postoffice;
use crate::{amqp, spawn_and_log};
use anyhow::Result;
use async_std::net::{SocketAddr, TcpListener};

use chrono::Utc;
use futures::TryStreamExt;
use kv_log_macro::{debug, info, trace};
use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, BasicQosOptions,
    ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions,
};
use lapin::types::FieldTable;
use lapin::{BasicProperties, ExchangeKind};
use uuid::Uuid;

mod docker;

// TODO - queues should be configurable for task routing
const TASK_QUEUE: &str = "waterwheel.tasks";

const RESULT_EXCHANGE: &str = "waterwheel.results";
const RESULT_QUEUE: &str = "waterwheel.results";

const HEARTBEAT_EXCHANGE: &str = "waterwheel.heartbeat";

pub async fn run_worker() -> Result<()> {
    amqp::amqp_connect().await?;
    postoffice::open()?;

    let max_tasks = std::env::var("WATERWHEEL_MAX_TASKS")?.parse::<u32>()?;

    for i in 0..max_tasks {
        spawn_and_log(&format!("worker-{}", i), process_work());
    }

    let mut app = tide::new();
    app.with(tide::log::LogMiddleware::new());
    app.at("/")
        .get(|_req| async { Ok("Hello from Waterwheel Worker!") });

    let host = std::env::var("WATERWHEEL_WORKER_ADDR").unwrap_or_else(|_| "127.0.0.1:0".to_owned());

    let tcp = TcpListener::bind(host).await?;
    let addr = tcp.local_addr()?;
    info!("worker listening on {}", addr);

    spawn_and_log("heatbeat", heartbeat(addr.clone()));

    app.listen(tcp).await?;

    Ok(())
}

async fn heartbeat(addr: SocketAddr) -> Result<!> {
    let chan = get_amqp_channel().await?;

    // declare outgoing exchange
    chan.exchange_declare(
        HEARTBEAT_EXCHANGE,
        ExchangeKind::Direct,
        ExchangeDeclareOptions {
            durable: false,
            ..ExchangeDeclareOptions::default()
        },
        FieldTable::default(),
    )
    .await?;

    let uuid = Uuid::new_v4();

    loop {
        trace!("posting heartbeat");
        chan.basic_publish(
            HEARTBEAT_EXCHANGE,
            "",
            BasicPublishOptions::default(),
            serde_json::to_vec(&WorkerHeartbeat {
                uuid,
                addr: addr.to_string(),
                last_seen_datetime: Utc::now(),
            })?,
            BasicProperties::default(),
        )
        .await?;

        async_std::task::sleep(std::time::Duration::from_secs(5)).await;
    }
}

async fn process_work() -> Result<!> {
    let chan = get_amqp_channel().await?;

    // declare queue for consuming incoming messages
    chan.queue_declare(
        TASK_QUEUE,
        QueueDeclareOptions {
            durable: true,
            ..QueueDeclareOptions::default()
        },
        FieldTable::default(),
    )
    .await?;

    // declare outgoing exchange and queue for progress reports
    chan.exchange_declare(
        RESULT_EXCHANGE,
        ExchangeKind::Direct,
        ExchangeDeclareOptions {
            durable: true,
            ..ExchangeDeclareOptions::default()
        },
        FieldTable::default(),
    )
    .await?;

    chan.queue_declare(
        RESULT_QUEUE,
        QueueDeclareOptions {
            durable: true,
            ..QueueDeclareOptions::default()
        },
        FieldTable::default(),
    )
    .await?;

    chan.queue_bind(
        RESULT_QUEUE,
        RESULT_EXCHANGE,
        "",
        QueueBindOptions::default(),
        FieldTable::default(),
    )
    .await?;

    chan.basic_qos(1, BasicQosOptions::default()).await?;

    let mut consumer = chan
        .basic_consume(
            TASK_QUEUE,
            "worker",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    while let Some((chan, msg)) = consumer.try_next().await? {
        let task_def: TaskDef = serde_json::from_slice(&msg.data)?;
        info!("received task", {
            task_id: task_def.task_id,
            trigger_datetime: task_def.trigger_datetime,
        });

        let success = if let Some(image) = task_def.image {
            docker::run_docker(image, task_def.args, task_def.env.unwrap_or_default()).await?
        } else {
            // task has no image, mark success immediately
            true
        };

        let result = match success {
            true => "success".to_string(),
            false => "failure".to_string(),
        };

        info!("task completed", {
            result: result,
            task_id: task_def.task_id,
            trigger_datetime: task_def.trigger_datetime,
        });

        let payload = serde_json::to_vec(&TaskResult {
            task_id: task_def.task_id,
            trigger_datetime: task_def.trigger_datetime,
            result,
        })?;

        chan.basic_publish(
            RESULT_EXCHANGE,
            "",
            BasicPublishOptions::default(),
            payload,
            BasicProperties::default(),
        )
        .await?;

        chan.basic_ack(msg.delivery_tag, BasicAckOptions::default())
            .await?;
        debug!("task acked");
    }

    unreachable!("consumer stopped consuming")
}
