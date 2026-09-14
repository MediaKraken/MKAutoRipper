mod drive_commands;
mod rabbit;

use amqprs::channel::{Channel, ConsumerMessage};
use discid::{DiscId, Features};
use serde_json::{json, Value};
use std::error::Error;
use std::ffi::OsStr;
use std::process::{self, Command, Stdio};
use sysinfo::{Signal, System};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::Notify;

fn ripper_queue_name() -> String {
    std::env::var("RIPPER_NAME").unwrap_or_else(|_| "ripper".to_string())
}

async fn handle_message(
    rabbit_channel: &Channel,
    msg: ConsumerMessage,
) -> Result<bool, Box<dyn Error + Send + Sync>> {
    let payload = msg
        .content
        .ok_or_else(|| "message had no payload".to_string())?;
    let json_message: Value = serde_json::from_slice(&payload)?;

    let mut exit_container = false;

    match json_message["Type"].as_str() {
        Some("idmedia") => {
            let uuid = json_message["UUID"].as_str().unwrap_or("");
            let mut media_id = json!({ "Blake3": null, "DiscID": null });

            if json_message["Media"] == "cd" {
                match DiscId::read_features(None, Features::ISRC) {
                    Ok(disc) => {
                        let id = disc.id();
                        media_id["DiscID"] = if id.is_empty() {
                            Value::Null
                        } else {
                            json!(id)
                        };
                    }
                    Err(err) => eprintln!("discid read failed: {err}"),
                }
            }

            let output = Command::new("sh")
                .args(["-c", "dd if=/dev/sr0 bs=4M | b3sum"])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()?;
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                media_id["Blake3"] = stdout
                    .split_whitespace()
                    .next()
                    .map(Value::from)
                    .unwrap_or(Value::Null);
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("disc hash failed: {stderr}");
            }

            let result = json!({
                "Type": "idmedia_result",
                "UUID": uuid,
                "Blake3": media_id["Blake3"],
                "DiscID": media_id["DiscID"],
            });
            rabbit::rabbitmq_publish(
                rabbit_channel,
                "mkterminal",
                serde_json::to_string(&result)?,
            )
            .await?;
        }
        Some("makemkv") => {
            let output = Command::new("makemkvcon")
                .args([
                    "--progress=-same",
                    "--cache=128",
                    "-r",
                    "mkv",
                    "disc:0",
                    format!("/{}/.", json_message["UUID"].as_str().unwrap_or("")).as_str(),
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("makemkvcon failed: {stderr}").into());
            }
        }
        Some("abcde") => {
            let output = Command::new("abcde")
                .args([
                    "-d",
                    "/dev/sr0",
                    format!("/{}/.", json_message["UUID"].as_str().unwrap_or("")).as_str(),
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("abcde failed: {stderr}").into());
            }
        }
        Some("stop") => {
            let s = System::new_all();
            for name in ["makemkvcon", "abcde"] {
                for pid in s.processes_by_name(OsStr::new(name)).map(|p| p.pid()) {
                    if let Some(process) = s.process(pid) {
                        if process.kill_with(Signal::Kill).is_none() {
                            eprintln!("kill signal not supported on this platform");
                        }
                    }
                }
            }
            exit_container = true;
        }
        Some("eject") => {
            drive_commands::drive_eject("/dev/sr0")?;
        }
        Some("close") => {
            drive_commands::drive_close("/dev/sr0")?;
        }
        Some(other) => eprintln!("unknown message type: {other}"),
        None => eprintln!("message missing Type"),
    }

    Ok(exit_container)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let queue_name = ripper_queue_name();
    let (_rabbit_connection, rabbit_channel) = rabbit::rabbitmq_connect(&queue_name).await?;
    let mut rabbit_consumer: UnboundedReceiver<ConsumerMessage> =
        rabbit::rabbitmq_consumer(&queue_name, &rabbit_channel).await?;
    let ack_channel = rabbit_channel.clone();

    tokio::spawn(async move {
        while let Some(msg) = rabbit_consumer.recv().await {
            let Some(deliver) = &msg.deliver else {
                continue;
            };
            let delivery_tag = deliver.delivery_tag();
            match handle_message(&ack_channel, msg).await {
                Ok(true) => {
                    let _ = rabbit::rabbitmq_ack(&ack_channel, delivery_tag).await;
                    eprintln!("rip stopped, exiting ripper container");
                    process::exit(1);
                }
                Ok(false) => {
                    let _ = rabbit::rabbitmq_ack(&ack_channel, delivery_tag).await;
                }
                Err(err) => {
                    eprintln!("failed handling message: {err}");
                    let _ = rabbit::rabbitmq_ack(&ack_channel, delivery_tag).await;
                }
            }
        }
    });

    let guard = Notify::new();
    guard.notified().await;
    Ok(())
}
