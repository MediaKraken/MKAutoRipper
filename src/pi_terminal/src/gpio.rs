use rppal::gpio::Gpio;
use std::error::Error;

pub fn gpio_set_pin(set_high: bool, pin_number: u8) -> Result<(), Box<dyn Error>> {
    let gpio = Gpio::new()?;
    let mut output = gpio.get(pin_number)?.into_output();

    output.set_reset_on_drop(false);

    if set_high {
        output.set_high();
    } else {
        output.set_low();
    }

    Ok(())
}