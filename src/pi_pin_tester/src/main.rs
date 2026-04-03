use rppal::gpio::{Gpio, Level};
use std::error::Error;
use tokio::time::{sleep, Duration};

// BCM pin numbering! Do not use physical pin numbers.
const GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT: u8 = 13; // physical 33

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let gpios = Gpio::new()?;

    // Most end stops are wired active-low with pull-up enabled.
    let pin_input = gpios
        .get(GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT)?
        .into_input_pullup();

    loop {
        if pin_input.read() == Level::Low {
            println!("Low");
            break;
        } else {
            println!("High");
        }

        sleep(Duration::from_millis(50)).await;
    }

    Ok(())
}