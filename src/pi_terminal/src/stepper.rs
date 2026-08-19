use crate::hardware_layout;
use ads1x1x::{
    channel,
    ic::{Ads1115, Resolution16Bit},
    Ads1x1x, TargetAddr,
};
use linux_embedded_hal::{I2CError, I2cdev};
use nb::block;
use rppal::gpio::{Gpio, Level};
use std::error::Error;
use std::fmt;
use std::thread;
use std::time::Duration;

const DEBOUNCE_READS: usize = 10;
const DEBOUNCE_THRESHOLD: f32 = 0.7;
const DEBOUNCE_DELAY_US: u64 = 15;

fn read_debounced(input: &rppal::gpio::InputPin, expected_level: Level) -> bool {
    let mut matches = 0usize;
    for _ in 0..DEBOUNCE_READS {
        if input.read() == expected_level {
            matches += 1;
        }
        thread::sleep(Duration::from_micros(DEBOUNCE_DELAY_US));
    }
    (matches as f32 / DEBOUNCE_READS as f32) >= DEBOUNCE_THRESHOLD
}

type Adc = Ads1x1x<I2cdev, Ads1115, Resolution16Bit, ads1x1x::mode::OneShot>;

#[derive(Debug)]
struct AdcError(ads1x1x::Error<I2CError>);

impl fmt::Display for AdcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            ads1x1x::Error::I2C(e) => write!(f, "ADS1115 I2C error: {e}"),
            ads1x1x::Error::InvalidInputData => f.write_str("ADS1115 invalid input data"),
        }
    }
}

impl Error for AdcError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.0 {
            ads1x1x::Error::I2C(e) => Some(e),
            ads1x1x::Error::InvalidInputData => None,
        }
    }
}

pub fn read(adc: &mut Adc) -> Result<i16, Box<dyn Error>> {
    let value = block!(adc.read(channel::SingleA0)).map_err(AdcError)?;
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
        // GPIO end-stop check (debounced)
        if hard_stop_pin_number != hardware_layout::GPIO_STEPPER_VERTICAL_END_STOP_ASSEMBLY {
            if read_debounced(&hard_stop_input, Level::Low) {
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