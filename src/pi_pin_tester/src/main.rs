use rppal::gpio::Gpio;
use rppal::i2c::I2c;
use rppal::pwm::{Channel, Pwm};
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use rppal::uart::{Parity, Uart};
use serde_json::{json, Value};
use std::error::Error;
use std::{cell::RefCell, rc::Rc};
use std::{fs, i32};
use tokio::time::{sleep, Duration};

mod gpio;

// BCM pin numbering! Do not use physcial pin numbers.
// Main movement arm
//const GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT: u8 = 25; // 22
const GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT: u8 = 13; // 33
const GPIO_STEPPER_HORIZONTAL_END_STOP_RIGHT: u8 = 21; // 40
const GPIO_STEPPER_HORIZONTAL_DIRECTION: u8 = 26; // 37
const GPIO_STEPPER_HORIZONTAL_PULSE: u8 = 19; // 35

// CD Picker/loader
const GPIO_STEPPER_VERTICAL_END_STOP_ASSEMBLY: u8 = 27; // 13
const GPIO_STEPPER_VERTICAL_END_STOP_BOTTOM: u8 = 22; // 15
const GPIO_STEPPER_VERTICAL_END_STOP_TOP: u8 = 10; // 19
const GPIO_STEPPER_VERTICAL_DIRECTION: u8 = 11; // 23
const GPIO_STEPPER_VERTICAL_PULSE: u8 = 9; // 21

// Image tray
const GPIO_STEPPER_TRAY_END_STOP_BACK: u8 = 17; // 11
const GPIO_STEPPER_TRAY_END_STOP_FRONT: u8 = 18; // 12
const GPIO_STEPPER_TRAY_DIRECTION: u8 = 24; // 18
const GPIO_STEPPER_TRAY_PULSE: u8 = 23; // 16

const GPIO_RELAY_VACUUM: u8 = 20; // 38
const GPIO_RELAY_LIGHT: u8 = 16; // 36

#[tokio::main]
pub async fn main() {
    let mut hard_stop: bool = false;
    let mut gpio_relay_vacuum_on: bool = false;

    let gpios = match Gpio::new() {
        Ok(gpios) => gpios,
        Err(msg) => panic!("Error: {}", msg),
    };
    // Retrieve a Pin without converting it to an InputPin,
    // OutputPin or IoPin, so we can check the pin's mode
    // and level without affecting its state.
    // let pin = gpios.get(GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT).unwrap();
    // while true {
    //     if pin.read() == rppal::gpio::Level::High {
    //         println!("High");
    //     } else {
    //         println!("Low");
    //     }
    // }
    let pin_input = gpios
        .get(GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT)
        .unwrap()
        .into_input_pullup();
    while true {
        if pin_input.is_high() {
            println!("High Input");
        } else {
            println!("Low Input");
        }
    }
}
