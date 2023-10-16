use std::process::{Command, Stdio};

// raspi-config - do NOT do
//  3
//  enable camera
//  reboot
// see if camera detectede
// vcgencmd get_camera

// raspi-config
//  updated raspi-config
// mine was two years old....
//  sudo rpi-update
//  sudo rpi-eeprom-update

// turn on i2c
// libcamera-hello --list-cameras
// grab image
// libcamera-still -o test.jpg
// works as this will allow ssh
// libcamera-hello --qt-preview
// libcamera-still -e png -o still-test.png -n

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