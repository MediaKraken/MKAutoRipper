use rppal::gpio::{Gpio, InputPin, Level};
use std::error::Error;
use std::thread;
use tokio::time::{sleep, Duration};

const DEBOUNCE_READS: usize = 10;
const DEBOUNCE_THRESHOLD: f32 = 0.7;
const DEBOUNCE_DELAY_US: u64 = 15;

fn read_debounced(input: &impl InputPin, expected_level: Level) -> bool {
    let mut matches = 0usize;
    for _ in 0..DEBOUNCE_READS {
        if input.read() == expected_level {
            matches += 1;
        }
        thread::sleep(Duration::from_micros(DEBOUNCE_DELAY_US));
    }
    (matches as f32 / DEBOUNCE_READS as f32) >= DEBOUNCE_THRESHOLD
}

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
        if read_debounced(&pin_input, Level::Low) {
            println!("Low");
            break;
        } else {
            println!("High");
        }

        sleep(Duration::from_millis(50)).await;
    }

    Ok(())
}