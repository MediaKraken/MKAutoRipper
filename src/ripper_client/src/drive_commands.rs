use std::process::{Command, Stdio};

pub async fn drive_eject(
    drive_path: &str,
) -> Result<String, std::io::Error> {
    let output = Command::new("eject")
        .args([
            &drive_path,
        ])
        .stdout(Stdio::piped())
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    Ok(stdout)
}

pub async fn drive_close(
    drive_path: &str,
) -> Result<String, std::io::Error> {
    let output = Command::new("eject")
        .args([
            "-t",
            &drive_path,
        ])
        .stdout(Stdio::piped())
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    Ok(stdout)
}

pub async fn drive_umount(
    drive_path: &str,
) -> Result<String, std::io::Error> {
    let output = Command::new("umount")
        .args([
            &drive_path,
        ])
        .stdout(Stdio::piped())
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    Ok(stdout)
}
