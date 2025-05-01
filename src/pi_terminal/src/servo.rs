use std::error::Error;
use std::thread;
use std::time::Duration;
use rppal::gpio::Gpio;

// Gpio uses BCM pin numbering. BCM GPIO 23 is tied to physical pin 16.
const GPIO_PWM: u8 = 23;

// Servo configuration. Change these values based on your servo's verified safe
// minimum and maximum values.

// Period: 20 ms (50 Hz). Pulse width: min. 1200 µs, neutral 1500 µs, max. 1800 µs.
const PERIOD_MS: u64 = 20;
const PULSE_MIN_US: u64 = 1200;
const PULSE_NEUTRAL_US: u64 = 1500;
const PULSE_MAX_US: u64 = 1800;

fn servo_move() -> Result<(), Box<dyn Error>> {
    // Retrieve the GPIO pin and configure it as an output.
    let mut pin = Gpio::new()?.get(GPIO_PWM)?.into_output();

    // Enable software-based PWM with the specified period, and rotate the servo by
    // setting the pulse width to its maximum value.
    pin.set_pwm(
        Duration::from_millis(PERIOD_MS),
        Duration::from_micros(PULSE_MAX_US),
    )?;

    // Sleep for 500 ms while the servo moves into position.
    thread::sleep(Duration::from_millis(500));

    // Rotate the servo to the opposite side.
    pin.set_pwm(
        Duration::from_millis(PERIOD_MS),
        Duration::from_micros(PULSE_MIN_US),
    )?;

    thread::sleep(Duration::from_millis(500));

    // Rotate the servo to its neutral (center) position in small steps.
    for pulse in (PULSE_MIN_US..=PULSE_NEUTRAL_US).step_by(10) {
        pin.set_pwm(
            Duration::from_millis(PERIOD_MS),
            Duration::from_micros(pulse),
        )?;
        thread::sleep(Duration::from_millis(20));
    }

    Ok(())
}