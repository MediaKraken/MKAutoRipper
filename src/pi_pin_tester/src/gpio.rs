use rppal::gpio::Gpio;
use std::error::Error;

// Gpio uses BCM pin numbering.
pub fn gpio_set_pin(set_high: bool, pin_number: u8) -> Result<(), Box<dyn Error>> {
    // create the GPIO object (mutable as we want to change the output)
    let gpios = match Gpio::new() {
        Ok(gpios) => gpios,
        Err(msg) => panic!("Error: {}", msg),
    };
    // retrieve the GPIO pin as an Output
    let mut output = match gpios.get(pin_number) {
        Ok(output) => output.into_output(),
        Err(msg) => panic!("Error: {}", msg),
    };
    if set_high {
        output.set_high();
    } else {
        output.set_low();
    }
    Ok(())
}
