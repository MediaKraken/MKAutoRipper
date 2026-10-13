#![no_std]
#![no_main]

use embedded_hal::digital::{InputPin, OutputPin};
use panic_halt as _;
use rp2040_hal::{
    clocks::init_clocks_and_plls,
    gpio::{FunctionPio0, Pins},
    pac::{self, PIO0},
    pio::{
        InstalledProgram, PIOBuilder, PIOExt, PinDir, Running, Rx, StateMachine,
        StateMachineIndex, Tx, UninitStateMachine,
    },
    sio::Sio,
    usb::UsbBus,
    watchdog::Watchdog,
};
use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::SerialPort;

// Required boot2 loader block for the RP2040 chip to boot from external flash
#[link_section = ".boot2"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

// ---------------------------------------------------------------------------
// Speed
// ---------------------------------------------------------------------------
// The PIO loop below takes 65 PIO cycles per step (32 high + 32 low + 1 jmp).
// PIO clock = 125 MHz / divider, so:  step_rate = 125e6 / divider / 65
//   divider 1923 -> ~1 kHz   (matches the old 500 us high / 500 us low)
//   divider  962 -> ~2 kHz
//   divider  192 -> ~10 kHz
const STEP_DIVIDER: u16 = 1923;

// STEP pins (must be set up as PIO0 pins). DIR / STOP are plain GPIO.
const STEP1: u8 = 2;
const STEP2: u8 = 5;
const STEP3: u8 = 8;

// ---------------------------------------------------------------------------
// Instruction helpers, used to peek at / reset a state machine from Rust
// ---------------------------------------------------------------------------
fn instr(operands: pio::InstructionOperands) -> pio::Instruction {
    pio::Instruction { operands, delay: 0, side_set: None }
}

fn mov_isr_x() -> pio::Instruction {
    instr(pio::InstructionOperands::MOV {
        destination: pio::MovDestination::ISR,
        op: pio::MovOperation::None,
        source: pio::MovSource::X,
    })
}

fn push_noblock() -> pio::Instruction {
    instr(pio::InstructionOperands::PUSH { if_full: false, block: false })
}

fn jmp_always(addr: u8) -> pio::Instruction {
    instr(pio::InstructionOperands::JMP {
        condition: pio::JmpCondition::Always,
        address: addr,
    })
}

fn set_pins_low() -> pio::Instruction {
    instr(pio::InstructionOperands::SET {
        destination: pio::SetDestination::PINS,
        data: 0,
    })
}

// ---------------------------------------------------------------------------
// Motor
// ---------------------------------------------------------------------------
struct Motor<SM: StateMachineIndex, D: OutputPin, I: InputPin> {
    sm: Option<StateMachine<(PIO0, SM), Running>>,
    tx: Tx<(PIO0, SM)>,
    rx: Rx<(PIO0, SM)>,
    dir_pin: D,
    stop_pin: I,
    stop_dir: bool, // direction that runs INTO this motor's limit switch
    offset: u8,     // where the PIO program starts in instruction memory
    busy: bool,
    steps: u32,
    clockwise: bool,
}

impl<SM: StateMachineIndex, D: OutputPin, I: InputPin> Motor<SM, D, I> {
    fn new(
        installed: InstalledProgram<PIO0>,
        uninit: UninitStateMachine<(PIO0, SM)>,
        step_pin: u8,
        dir_pin: D,
        stop_pin: I,
        stop_dir: bool,
    ) -> Self {
        let offset = installed.offset();
        let (mut sm, rx, tx) = PIOBuilder::from_installed_program(installed)
            .set_pins(step_pin, 1)
            .clock_divisor_fixed_point(STEP_DIVIDER, 0)
            .build(uninit);
        sm.set_pindirs([(step_pin, PinDir::Output)]);
        Motor {
            sm: Some(sm.start()),
            tx,
            rx,
            dir_pin,
            stop_pin,
            stop_dir,
            offset,
            busy: false,
            steps: 0,
            clockwise: true,
        }
    }

    /// Kick off a move. Returns false if the motor is already moving.
    fn start_move(&mut self, steps: u32, clockwise: bool) -> bool {
        if self.busy {
            return false;
        }
        // Throw away any stale completion words
        while self.rx.read().is_some() {}

        if clockwise {
            let _ = self.dir_pin.set_high();
        } else {
            let _ = self.dir_pin.set_low();
        }
        cortex_m::asm::delay(250); // ~2 us DIR setup time at 125 MHz

        self.steps = steps;
        self.clockwise = clockwise;
        self.busy = true;
        // jmp x-- runs the loop body once more than the count, hence -1
        self.tx.write(steps - 1);
        true
    }

    /// Call from the main loop. Returns Some((steps_moved, hit_stop)) when a move ends.
    fn service(&mut self) -> Option<(u32, bool)> {
        if !self.busy {
            return None;
        }

        // The PIO program pushes a word to the RX FIFO when it finishes
        if self.rx.read().is_some() {
            self.busy = false;
            return Some((self.steps, false));
        }

        // Only check the limit switch when moving toward it, so you can back off it
        if self.clockwise == self.stop_dir && self.stop_pin.is_low().unwrap_or(false) {
            let moved = self.abort();
            self.busy = false;
            return Some((moved, true));
        }
        None
    }

    /// Stop the state machine mid-move, work out how far it got, and reset it.
    fn abort(&mut self) -> u32 {
        let mut sm = self.sm.take().unwrap().stop();

        // Copy X (steps left) into the RX FIFO. If a "done" word is already
        // queued, the last value read is still correct (X = 0xFFFFFFFF after a finish).
        sm.exec_instruction(mov_isr_x());
        sm.exec_instruction(push_noblock());
        let mut x = None;
        while let Some(v) = self.rx.read() {
            x = Some(v);
        }
        let remaining = x.unwrap_or(0).wrapping_add(1).min(self.steps);

        // Reset the state machine back to the top of the program, STEP low
        sm.restart();
        sm.clear_fifos();
        sm.exec_instruction(jmp_always(self.offset));
        sm.exec_instruction(set_pins_low());
        self.sm = Some(sm.start());

        self.steps - remaining
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------
#[cortex_m_rt::entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    const XTAL_FREQ_HZ: u32 = 12_000_000;
    let clocks = init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let pins = Pins::new(pac.IO_BANK0, pac.PADS_BANK0, sio.gpio_bank0, &mut pac.RESETS);

    // USB serial
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
        .device_class(2) // CDC
        .build();

    // PIO program: wait for a count, emit that many STEP pulses, then signal done.
    let program = pio_proc::pio_asm!(
        "pull block",         // sleep until the CPU pushes a step count
        "mov x, osr",
        "pulse:",
        "set pins, 1 [31]",   // STEP high
        "set pins, 0 [31]",   // STEP low
        "jmp x-- pulse",
        "mov isr, null",
        "push noblock",       // tell the CPU we're done
    );
    let (mut pio, sm0, sm1, sm2, _sm3) = pac.PIO0.split(&mut pac.RESETS);
    let installed = pio.install(&program.program).unwrap();

    // Hand the STEP pins over to PIO0
    let _s1 = pins.gpio2.into_function::<FunctionPio0>();
    let _s2 = pins.gpio5.into_function::<FunctionPio0>();
    let _s3 = pins.gpio8.into_function::<FunctionPio0>();

    // All three state machines share the one installed copy of the program.
    // SAFETY: the program is never uninstalled while the machines run.
    let mut motor1 = Motor::new(
        unsafe { installed.share() },
        sm0,
        STEP1,
        pins.gpio3.into_push_pull_output(),
        pins.gpio4.into_pull_up_input(),
        true,
    );
    let mut motor2 = Motor::new(
        unsafe { installed.share() },
        sm1,
        STEP2,
        pins.gpio6.into_push_pull_output(),
        pins.gpio7.into_pull_up_input(),
        true,
    );
    let mut motor3 = Motor::new(
        installed,
        sm2,
        STEP3,
        pins.gpio9.into_push_pull_output(),
        pins.gpio10.into_pull_up_input(),
        true,
    );

    let mut buf = [0u8; 64];
    let mut line_buf = [0u8; 32];
    let mut line_len = 0usize;
    let mut overflow = false;

    loop {
        // USB must be polled constantly; nothing blocks any more.
        let _ = usb_dev.poll(&mut [&mut serial]);

        if let Ok(count) = serial.read(&mut buf) {
            for &byte in &buf[..count] {
                if byte == b'\n' || byte == b'\r' {
                    if overflow {
                        reply(&mut serial, b"ERR", None);
                    } else if line_len > 0 {
                        match parse_command(&line_buf[..line_len]) {
                            Some((id, steps, cw)) if (1..=3).contains(&id) && steps > 0 => {
                                let started = match id {
                                    1 => motor1.start_move(steps, cw),
                                    2 => motor2.start_move(steps, cw),
                                    _ => motor3.start_move(steps, cw),
                                };
                                if !started {
                                    reply(&mut serial, b"BUSY", None);
                                }
                            }
                            Some((id, 0, _)) if (1..=3).contains(&id) => {
                                reply(&mut serial, b"DONE", Some((id as u32, 0)));
                            }
                            _ => reply(&mut serial, b"ERR", None),
                        }
                    }
                    line_len = 0;
                    overflow = false;
                } else if line_len < line_buf.len() {
                    line_buf[line_len] = byte;
                    line_len += 1;
                } else {
                    overflow = true; // line too long: reject it instead of parsing a truncated one
                }
            }
        }

        // Check every motor for completion or a limit-switch hit
        if let Some((moved, stopped)) = motor1.service() {
            reply(&mut serial, if stopped { b"STOPPED" } else { b"DONE" }, Some((1, moved)));
        }
        if let Some((moved, stopped)) = motor2.service() {
            reply(&mut serial, if stopped { b"STOPPED" } else { b"DONE" }, Some((2, moved)));
        }
        if let Some((moved, stopped)) = motor3.service() {
            reply(&mut serial, if stopped { b"STOPPED" } else { b"DONE" }, Some((3, moved)));
        }
    }
}

// ---------------------------------------------------------------------------
// Serial helpers
// ---------------------------------------------------------------------------

/// Sends  TAG\n  or  TAG,motor,steps_moved\n
fn reply(serial: &mut SerialPort<'_, UsbBus>, tag: &[u8], info: Option<(u32, u32)>) {
    let mut out = [0u8; 40];
    let mut n = 0;
    for &b in tag {
        out[n] = b;
        n += 1;
    }
    if let Some((motor, moved)) = info {
        out[n] = b',';
        n += 1;
        n += write_u32(&mut out[n..], motor);
        out[n] = b',';
        n += 1;
        n += write_u32(&mut out[n..], moved);
    }
    out[n] = b'\n';
    n += 1;
    // Small message, so a single write normally fits the USB buffer.
    let _ = serial.write(&out[..n]);
}

fn write_u32(out: &mut [u8], mut v: u32) -> usize {
    let mut tmp = [0u8; 10];
    let mut i = 0;
    loop {
        tmp[i] = b'0' + (v % 10) as u8;
        v /= 10;
        i += 1;
        if v == 0 {
            break;
        }
    }
    for j in 0..i {
        out[j] = tmp[i - 1 - j];
    }
    i
}

// Command format:  motor,steps,dir\n   e.g.  1,2000,1
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

    let motor_id = parse_u32(&input[..c1])?;
    let steps = parse_u32(&input[c1 + 1..c2])?;
    let dir_val = parse_u32(&input[c2 + 1..])?;

    if motor_id > 255 {
        return None;
    }
    Some((motor_id as u8, steps, dir_val == 1))
}

fn parse_u32(s: &[u8]) -> Option<u32> {
    if s.is_empty() {
        return None;
    }
    let mut val = 0u32;
    for &b in s {
        if !b.is_ascii_digit() {
            return None;
        }
        val = val.checked_mul(10)?.checked_add((b - b'0') as u32)?;
    }
    Some(val)
}