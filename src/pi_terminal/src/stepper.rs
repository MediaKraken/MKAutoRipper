use rppal::gpio::Gpio;
use std::error::Error;
use std::thread;
use std::time::Duration;

// Gpio uses BCM pin numbering.
pub fn gpio_stepper_move(
    steps_to_take: u64,
    pulse_pin_number: u8,
    direction_pin_number: u8,
    move_clockwise: bool,
) -> Result<(), Box<dyn Error>> {
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
    // set direction
    if move_clockwise {
        stepper_direction_output.set_high();
    } else {
        stepper_direction_output.set_low();
    }
    // move the number of steps
    for _step_num in 0..steps_to_take {
        stepper_pulse_output.set_high();
        thread::sleep(Duration::from_micros(500));
        stepper_pulse_output.set_low();
        thread::sleep(Duration::from_micros(500));
        // TODO check for hardstops
    }
    Ok(())
}
