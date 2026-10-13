#![no_std]
#![no_main]

use panic_halt as _;
use rp2040_hal::{pac, entry};

#[link_section = ".boot2"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[entry]
fn main() -> ! {
    // Just loop forever and do nothing
    loop {
        cortex_m::asm::nop();
    }
}