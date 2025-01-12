use std::process::{Command, Stdio};

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

pub fn camera_take_image(
    media_file: &str,
) -> Result<String, std::io::Error> {
    let output = Command::new("libcamera-still")
        .args([
            "-e",
            "png",
            "-o",
            &media_file,
            "-n",
        ])
        .stdout(Stdio::piped())
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    Ok(stdout)
}