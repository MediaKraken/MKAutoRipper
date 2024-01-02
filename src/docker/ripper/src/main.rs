mod drive_commands;
use serde_json::Value;
use std::error::Error;
use std::process::{Command, Stdio};
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
                if json_message["Type"] == "makemkv" {
                    // begin makemkv rip
                    let output = Command::new("makemkvcon")
                        .args([
                            "--progress=-same",
                            "--cache=128",
                            "-r",
                            "mkv",
                            "disc:0",
                            "/ripoutput/.",
                        ])
                        .stdout(Stdio::piped())
                        .output()
                        .unwrap();
                    let stdout = String::from_utf8(output.stdout).unwrap();
                    let json_output: Value = serde_json::from_str(&stdout).unwrap();
                } else if json_message["Type"] == "abcde" {
                    // TODO begin abcde rip
                } else if json_message["Type"] == "stop" {
                    // TODO stop rip
                } else if json_message["Type"] == "eject" {
                    // eject media
                    let _results = drive_commands::drive_eject("/dev/cdrom").await;
                }

                let _result =
                    rabbit::rabbitmq_ack(&rabbit_channel, msg.deliver.unwrap().delivery_tag())
                        .await;
            }
            sleep(Duration::from_secs(1)).await;
        }
    });
    let guard = Notify::new();
    guard.notified().await;
    Ok(())
}
