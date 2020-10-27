#![no_std]

use defmt_rtt as _; // global logger
use panic_probe as _; // panic handler

// Re-export dependencies, so test suite can use them.
pub extern crate defmt;
pub extern crate lpc8xx_hal;
pub extern crate step_dir;

use lpc8xx_hal::cortex_m::asm;

/// Causes probe-run to exit with exit code 0
pub fn exit() -> ! {
    loop {
        asm::bkpt();
    }
}
