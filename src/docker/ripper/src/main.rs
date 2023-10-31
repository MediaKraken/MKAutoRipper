use serde_json::Value;
use std::error::Error;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::Notify;
use tokio::time::{sleep, Duration};

mod rabbit;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (_rabbit_connection, rabbit_channel) = rabbit::rabbitmq_connect("ripper").await.unwrap();

    let mut rabbit_consumer = rabbit::rabbitmq_consumer("ripper", &rabbit_channel)
        .await
        .unwrap();

    tokio::spawn(async move {
        while let Some(msg) = rabbit_consumer.recv().await {
            if let Some(payload) = msg.content {
                let json_message: Value =
                    serde_json::from_str(&String::from_utf8_lossy(&payload)).unwrap();
                // perform command sent to ripper

                // TODO begin makemkv rip
                // TODO begin abcde rip
                // TODO stop rip

                let _result =
                    rabbit::rabbitmq_ack(&rabbit_channel, msg.deliver.unwrap().delivery_tag())
                        .await;
            }
        }
    });
    let guard = Notify::new();
    guard.notified().await;
    Ok(())
}
