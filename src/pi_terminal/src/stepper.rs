use rppal::gpio::{Gpio, Level, Trigger};
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const HARD_STOP_DEBOUNCE: Duration = Duration::from_millis(20);
const HARD_STOP_POLL_TIMEOUT: Duration = Duration::from_millis(10);

// Gpio uses BCM pin numbering.
pub fn gpio_stepper_move(
    steps_to_take: i32,
    pulse_pin_number: u8,
    direction_pin_number: u8,
    hard_stop_pin_number: u8,
    move_clockwise: bool,
    motor_speed: u64,
) -> Result<i32, Box<dyn Error>> {
    let gpios = Gpio::new()?;

    let mut stepper_pulse_output = gpios.get(pulse_pin_number)?.into_output();
    let mut stepper_direction_output = gpios.get(direction_pin_number)?.into_output();
    let mut hard_stop_input = gpios.get(hard_stop_pin_number)?.into_input_pullup();
    if move_clockwise {
        stepper_direction_output.set_high();
    } else {
        stepper_direction_output.set_low();
    }

    // A dedicated thread watches the hard stop line via a kernel GPIO
    // interrupt and raises this flag on a (debounced) falling edge.
    let hard_stopped = Arc::new(AtomicBool::new(false));
    let stop_watcher = Arc::new(AtomicBool::new(false));

    let mut stop_thread = None;

    // End stops are active-low. If already pressed, stop immediately.
    if hard_stop_input.read() == Level::Low {
        hard_stopped.store(true, Ordering::Relaxed);
    } else {
        hard_stop_input
            .set_interrupt(Trigger::FallingEdge, Some(HARD_STOP_DEBOUNCE))?;
        let flag = Arc::clone(&hard_stopped);
        let done = Arc::clone(&stop_watcher);
        stop_thread = Some(thread::spawn(move || {
            let mut pin = hard_stop_input;
            loop {
                if done.load(Ordering::Relaxed) {
                    break;
                }
                match pin.poll_interrupt(false, Some(HARD_STOP_POLL_TIMEOUT)) {
                    Ok(Some(_event)) => {
                        flag.store(true, Ordering::Relaxed);
                        break;
                    }
                    Ok(None) => {}
                    Err(err) => {
                        eprintln!("hard stop interrupt poll failed: {err}");
                        break;
                    }
                }
            }
            let _ = pin.clear_interrupt();
        }));
    }

    let mut steps_moved = 0;

    for _ in 0..steps_to_take {
        if hard_stopped.load(Ordering::Relaxed) {
            println!("Hard Stop");
            break;
        }

        stepper_pulse_output.set_high();
        thread::sleep(Duration::from_micros(motor_speed));
        stepper_pulse_output.set_low();
        thread::sleep(Duration::from_micros(motor_speed));

        steps_moved += 1;
    }

    stop_watcher.store(true, Ordering::Relaxed);
    if let Some(handle) = stop_thread {
        let _ = handle.join();
    }

    Ok(steps_moved)
}
