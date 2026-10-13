#![no_std]
#![no_main]

use embedded_hal::digital::{InputPin, OutputPin};
use panic_halt as _;
use rp2040_hal::{
    clocks::init_clocks_and_plls,
    pac,
    sio::Sio,
    watchdog::Watchdog,
    usb::UsbBus,
    Clock,
};
use usb_device::{class_prelude::*, prelude::*, device::UsbVidPid};
use usbd_serial::SerialPort;
use cortex_m::interrupt;

// Required boot2 loader block for the RP2040 chip to boot from external flash
#[link_section = ".boot2"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

struct Motor<S: OutputPin, D: OutputPin, I: InputPin> {
    step_pin: S,
    dir_pin: D,
    stop_pin: I,
}

impl<S: OutputPin, D: OutputPin, I: InputPin> Motor<S, D, I> {
    pub fn step_motor(&mut self, steps: u32, clockwise: bool, delay_us: u32, delay: &mut cortex_m::delay::Delay) -> (u32, bool) {
        if clockwise {
            let _ = self.dir_pin.set_high();
        } else {
            let _ = self.dir_pin.set_low();
        }

        let mut steps_moved = 0;
        let mut hit_hard_stop = false;

        for _ in 0..steps {
            if self.stop_pin.is_low().unwrap_or(false) {
                hit_hard_stop = true;
                break;
            }

            let _ = self.step_pin.set_high();
            delay.delay_us(delay_us);
            let _ = self.step_pin.set_low();
            delay.delay_us(delay_us);

            steps_moved += 1;
        }

        (steps_moved, hit_hard_stop)
    }
}

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    const XTAL_FREQ_HZ: u32 = 12_000_000;
    let mut clocks = init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    ).ok().unwrap();

    let pins = rp2040_hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Setup USB Serial with correct RP2040 USB peripheral registers
    let usb_bus = UsbBusAllocator::new(UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));
    let mut serial = SerialPort::new(&usb_bus);
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .strings(&[StringDescriptors::default()
            .manufacturer("SelfHosted")
            .product("StepperController")
            .serial_number("STEPS01")])
        .unwrap()
        .device_class(2) // 2 represents CDC/Serial class
        .build();

    // Motor 1 (Pins 2, 3, 4)
    let mut motor1 = Motor {
        step_pin: pins.gpio2.into_push_pull_output(),
        dir_pin: pins.gpio3.into_push_pull_output(),
        stop_pin: pins.gpio4.into_pull_up_input(),
    };

    // Motor 2 (Pins 5, 6, 7)
    let mut motor2 = Motor {
        step_pin: pins.gpio5.into_push_pull_output(),
        dir_pin: pins.gpio6.into_push_pull_output(),
        stop_pin: pins.gpio7.into_pull_up_input(),
    };

    // Motor 3 (Pins 8, 9, 10)
    let mut motor3 = Motor {
        step_pin: pins.gpio8.into_push_pull_output(),
        dir_pin: pins.gpio9.into_push_pull_output(),
        stop_pin: pins.gpio10.into_pull_up_input(),
    };

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    let default_speed_us = 500;

    let mut buf = [0u8; 64];
    let mut line_buf = [0u8; 32];
    let mut line_len = 0;

    loop {
        if usb_dev.poll(&mut [&mut serial]) {
            match serial.read(&mut buf) {
                Ok(count) if count > 0 => {
                    for &byte in &buf[..count] {
                        if byte == b'\n' || byte == b'\r' {
                            if line_len > 0 {
                                if let Some((motor_id, steps, dir)) = parse_command(&line_buf[..line_len]) {
                                    let (_moved, stopped) = match motor_id {
                                        1 => motor1.step_motor(steps, dir, default_speed_us, &mut delay),
                                        2 => motor2.step_motor(steps, dir, default_speed_us, &mut delay),
                                        3 => motor3.step_motor(steps, dir, default_speed_us, &mut delay),
                                        _ => (0, false),
                                    };
                                    let msg: &[u8] = if stopped { b"STOPPED\n" } else { b"DONE\n" };
                                    let _ = serial.write(msg);
                                }
                                line_len = 0;
                            }
                        } else if line_len < line_buf.len() {
                            line_buf[line_len] = byte;
                            line_len += 1;
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn parse_command(input: &[u8]) -> Option<(u8, u32, bool)> {
    let mut comma1 = None;
    let mut comma2 = None;

    for (i, &b) in input.iter().enumerate() {
        if b == b',' {
            if comma1.is_none() {
                comma1 = Some(i);
            } else if comma2.is_none() {
                comma2 = Some(i);
                break;
            }
        }
    }

    let c1 = comma1?;
    let c2 = comma2?;

    let motor_id = parse_u8(&input[..c1])?;
    let steps = parse_u32(&input[c1 + 1..c2])?;
    let dir_val = parse_u8(&input[c2 + 1..])?;

    Some((motor_id, steps, dir_val == 1))
}

fn parse_u8(s: &[u8]) -> Option<u8> {
    let mut val = 0u8;
    if s.is_empty() { return None; }
    for &b in s {
        if !(b'0'..=b'9').contains(&b) { return None; }
        val = val.checked_mul(10)?.checked_add((b - b'0') as u8)?;
    }
    Some(val)
}

fn parse_u32(s: &[u8]) -> Option<u32> {
    let mut val = 0u32;
    if s.is_empty() { return None; }
    for &b in s {
        if !(b'0'..=b'9').contains(&b) { return None; }
        val = val.checked_mul(10)?.checked_add((b - b'0') as u32)?;
    }
    Some(val)
}