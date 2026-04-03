use std::path::Path;
use std::process::Command;

// raspi-config
//  updated raspi-config
// mine was two years old....
//  sudo rpi-update
//  sudo reboot
//  sudo rpi-eeprom-update
//  sudo reboot

// turn on i2c
// libcamera-hello --list-cameras
// grab image
// libcamera-still -o test.jpg
// works as this will allow ssh
// libcamera-hello --qt-preview
// libcamera-still -e png -o still-test.png -n
// show image stats
// file still-test.png

pub fn camera_take_image(media_file: &str) -> Result<(), std::io::Error> {
    let output = Command::new("libcamera-still")
        .args(["-e", "png", "-o", media_file, "-n"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(std::io::Error::other(format!(
            "libcamera-still failed: {}",
            stderr.trim()
        )));
    }

    if !Path::new(media_file).exists() {
        return Err(std::io::Error::other(format!(
            "output file was not created: {}",
            media_file
        )));
    }

    Ok(())
}