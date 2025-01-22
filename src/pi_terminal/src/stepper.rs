use rppal::gpio::{Gpio, Level};
use std::error::Error;
use std::thread;
use std::time::Duration;

// Gpio uses BCM pin numbering.
pub fn gpio_stepper_move(
    steps_to_take: i32,
    pulse_pin_number: u8,
    direction_pin_number: u8,
    hard_stop_pin_number: u8,
    move_clockwise: bool,
) -> Result<i32, Box<dyn Error>> {
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
    // Retrieve a Pin without converting it to an InputPin,
    // OutputPin or IoPin, so we can check the pin's mode
    // and level without affecting its state.
    let pin = gpios.get(hard_stop_pin_number)?;
//    let pin = gpios.get(hard_stop_pin_number)?.into_input_pulldown();
    // set direction
    if move_clockwise {
        stepper_direction_output.set_high();
    } else {
        stepper_direction_output.set_low();
    }
    // move the number of steps
    for _step_num in 0..steps_to_take {
        steps_moved += 1;
        stepper_pulse_output.set_high();
        thread::sleep(Duration::from_micros(500));
        stepper_pulse_output.set_low();
        thread::sleep(Duration::from_micros(500));
        // Check for hardstops
        if pin.read() == rppal::gpio::Level::High {
//        if pin.is_high() {
            println!("Hard Stop");
            break;
        }
    }
    Ok(steps_moved)
}
