use crate::hardware_layout;
use rppal::gpio::Gpio;
use std::error::Error;
use std::thread;
use std::time::Duration;
use linux_embedded_hal::I2cdev;
use nb::block;
use ads1x1x::{
    channel,
    ic::{Ads1115, Resolution16Bit},
    Ads1x1x, TargetAddr,
};

type Adc = Ads1x1x<I2cdev, Ads1115, Resolution16Bit, ads1x1x::mode::OneShot>;

pub fn read(adc: &mut Adc) -> i16 {
    block!(adc.read(channel::SingleA0)).unwrap_or(0)
}

// Gpio uses BCM pin numbering.
pub fn gpio_stepper_move(
    steps_to_take: i32,
    pulse_pin_number: u8,
    direction_pin_number: u8,
    hard_stop_pin_number: u8,
    move_clockwise: bool,
    motor_speed: u64,
) -> Result<i32, Box<dyn Error>> {
    let dev = I2cdev::new("/dev/i2c-1").unwrap();
    let mut adc = Ads1x1x::new_ads1115(dev, TargetAddr::default());
    let mut steps_moved: i32 = 0;
    let gpios = match Gpio::new() {
        Ok(gpios) => gpios,
        Err(msg) => panic!("Error: {}", msg),
    };
    // retrieve the GPIO pin as an Output
    let mut stepper_pulse_output = match gpios.get(pulse_pin_number) {
        Ok(stepper_pulse_output) => stepper_pulse_output.into_output(),
        Err(msg) => panic!("Error: {}", msg),
    };
    let mut stepper_direction_output = match gpios.get(direction_pin_number) {
        Ok(stepper_direction_output) => stepper_direction_output.into_output(),
        Err(msg) => panic!("Error: {}", msg),
    };
    let pin = gpios.get(hard_stop_pin_number).unwrap();
    // set direction
    if move_clockwise {
        stepper_direction_output.set_high();
    } else {
        stepper_direction_output.set_low();
    }
    // move the number of steps
    let mut pin_check = 0;
    let mut height_sensor_data = [0u8; 1];
    for _step_num in 0..steps_to_take {
        steps_moved += 1;
        pin_check += 1;
        stepper_pulse_output.set_high();
        thread::sleep(Duration::from_micros(motor_speed));
        stepper_pulse_output.set_low();
        thread::sleep(Duration::from_micros(motor_speed));
        // Check for hard stops
        if hard_stop_pin_number == hardware_layout::GPIO_STEPPER_VERTICAL_END_STOP_ASSEMBLY {
            let value = read(&mut adc);
            if value < 16000 {
                println!("Sensor Stop");
                break;
            }
        }
        // if pin_check == 10 {
        //     pin_check = 0;
        //     if pin.read() == rppal::gpio::Level::Low {
        //         println!("Hard Stop");
        //         break;
        //     }
        // }
    }
    let _dev = adc.destroy_ads1115();
    Ok(steps_moved)
}
