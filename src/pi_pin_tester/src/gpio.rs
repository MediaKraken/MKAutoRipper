use rppal::gpio::Gpio;
use std::error::Error;

// Gpio uses BCM pin numbering.
pub fn gpio_set_pin(set_high: bool, pin_number: u8) -> Result<(), Box<dyn Error>> {
    let gpios = Gpio::new()?;
    let mut output = gpios.get(pin_number)?.into_output();
    output.write(if set_high {
        rppal::gpio::Level::High
    } else {
        rppal::gpio::Level::Low
    });
    Ok(())
}