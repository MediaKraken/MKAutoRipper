use rppal::gpio::Gpio;
use std::error::Error;
use std::thread;
use std::time::Duration;

// BCM numbering
const GPIO_PWM: u8 = 23;

// 50 Hz servo timing
const PERIOD_MS: u64 = 20;
const PULSE_MIN_US: u64 = 1200;
const PULSE_NEUTRAL_US: u64 = 1500;
const PULSE_MAX_US: u64 = 1800;

fn set_servo_pulse(
    pin: &mut rppal::gpio::OutputPin,
    pulse_us: u64,
) -> Result<(), Box<dyn Error>> {
    pin.set_pwm(
        Duration::from_millis(PERIOD_MS),
        Duration::from_micros(pulse_us),
    )?;
    Ok(())
}

fn servo_move() -> Result<(), Box<dyn Error>> {
    let mut pin = Gpio::new()?.get(GPIO_PWM)?.into_output();

    // Move to max
    set_servo_pulse(&mut pin, PULSE_MAX_US)?;
    thread::sleep(Duration::from_millis(500));

    // Move to min
    set_servo_pulse(&mut pin, PULSE_MIN_US)?;
    thread::sleep(Duration::from_millis(500));

    // Sweep back to center smoothly
    for pulse in (PULSE_MIN_US..=PULSE_NEUTRAL_US).step_by(10) {
        set_servo_pulse(&mut pin, pulse)?;
        thread::sleep(Duration::from_millis(20));
    }

    // Hold center briefly
    thread::sleep(Duration::from_millis(250));

    // Stop PWM so the servo is not constantly driven
    pin.clear_pwm()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    servo_move()
}