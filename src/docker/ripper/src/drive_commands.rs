use std::process::{Command, Stdio};

pub fn drive_eject(drive_path: &str) -> Result<String, std::io::Error> {
    let output = Command::new("eject")
        .args([drive_path])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(std::io::Error::other(format!("eject failed: {stderr}")));
    }
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    Ok(stdout)
}

pub fn drive_close(drive_path: &str) -> Result<String, std::io::Error> {
    let output = Command::new("eject")
        .args(["-t", drive_path])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(std::io::Error::other(format!(
            "tray close failed: {stderr}"
        )));
    }
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    Ok(stdout)
}
