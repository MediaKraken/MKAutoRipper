use crate::hardware_layout;
use ads1x1x::{
    channel,
    ic::{Ads1115, Resolution16Bit},
    Ads1x1x, TargetAddr,
};
use linux_embedded_hal::I2cdev;
use nb::block;
use rppal::gpio::{Gpio, Level};
use std::error::Error;
use std::thread;
use std::time::Duration;

type Adc = Ads1x1x<I2cdev, Ads1115, Resolution16Bit, ads1x1x::mode::OneShot>;

pub fn read(adc: &mut Adc) -> Result<i16, Box<dyn Error>> {
    let value = block!(adc.read(channel::SingleA0))?;
    Ok(value)
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
    let dev = I2cdev::new("/dev/i2c-1")?;
    let mut adc = Ads1x1x::new_ads1115(dev, TargetAddr::default());

    let gpios = Gpio::new()?;

    let mut stepper_pulse_output = gpios.get(pulse_pin_number)?.into_output();
    let mut stepper_direction_output = gpios.get(direction_pin_number)?.into_output();
    let hard_stop_input = gpios.get(hard_stop_pin_number)?.into_input();

    if move_clockwise {
        stepper_direction_output.set_high();
    } else {
        stepper_direction_output.set_low();
    }

    let mut steps_moved = 0;

    for _ in 0..steps_to_take {
        // GPIO end-stop check
        if hard_stop_pin_number != hardware_layout::GPIO_STEPPER_VERTICAL_END_STOP_ASSEMBLY {
            if hard_stop_input.read() == Level::Low {
                println!("Hard Stop");
                break;
            }
        } else {
            // ADS1115 sensor stop check
            let value = read(&mut adc)?;
            if value < 16000 {
                println!("Sensor Stop");
                break;
            }
        }

        stepper_pulse_output.set_high();
        thread::sleep(Duration::from_micros(motor_speed));
        stepper_pulse_output.set_low();
        thread::sleep(Duration::from_micros(motor_speed));

        steps_moved += 1;
    }

    let _ = adc.destroy_ads1115();

    Ok(steps_moved)
}