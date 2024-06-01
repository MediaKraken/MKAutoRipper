mod drive_commands;
use serde_json::Value;
use std::error::Error;
use std::process::{exit, Command, Stdio};
use sysinfo::{Components, Disks, Networks, Pid, Signal, System, Users};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::Notify;
use tokio::time::{sleep, Duration};
mod rabbit;
use std::process;

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
                let mut exit_container: bool = false;
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
                    // begin abcde rip
                    let output = Command::new("abcde")
                        .args([
                            "-d",
                            "/dev/cdrom",
                            "/ripoutput/.",
                        ])
                        .stdout(Stdio::piped())
                        .output()
                        .unwrap();
                    let stdout = String::from_utf8(output.stdout).unwrap();
                    let json_output: Value = serde_json::from_str(&stdout).unwrap();
                    // TODO look for "Finished."
                } else if json_message["Type"] == "stop" {
                    // stop rip
                    let s = System::new_all();
                    for process in s.processes_by_name("makemkvcon") {
                        if let Some(process) = s.process(process.pid()) {
                            if process.kill_with(Signal::Kill).is_none() {
                                println!("This signal isn't supported on this platform");
                            }
                        }
                    }
                    for process in s.processes_by_name("abcde") {
                        if let Some(process) = s.process(process.pid()) {
                            if process.kill_with(Signal::Kill).is_none() {
                                println!("This signal isn't supported on this platform");
                            }
                        }
                    }
                    exit_container = true;
                } else if json_message["Type"] == "eject" {
                    // eject media
                    let _results = drive_commands::drive_eject("/dev/cdrom").await;
                }

                let _result =
                    rabbit::rabbitmq_ack(&rabbit_channel, msg.deliver.unwrap().delivery_tag())
                        .await;
                if exit_container == true {
                    // Exit program and stop container
                    process::exit(0x0100);
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
    });
    let guard = Notify::new();
    guard.notified().await;
    Ok(())
}
