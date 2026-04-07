//! Architecture-independent kernel logic for RINO.
//!
//! **THE RULE**: This crate must NEVER contain any architecture-specific code.
//! No x86 instructions, no ARM registers, no RISC-V CSRs. If you need
//! hardware access, add a method to a trait in `hal-traits` and implement
//! it in the appropriate `arch-*` crate.
//!
//! Everything here works on any platform that implements `hal_traits::Platform`.

#![no_std]

use hal_traits::{Platform, SerialPort};

/// The main kernel entry point.
///
/// Called by the architecture-specific boot code after hardware init.
/// This function is generic over `P: Platform` — it works identically
/// on x86_64, AArch64, or any future platform.
///
/// The `Platform` trait gives us access to a serial port (and later,
/// a scheduler, memory manager, interrupt controller, etc.)
pub fn kernel_main<P: Platform>(mut platform: P) -> ! {
    let serial = platform.serial();

    serial.write_str("=============================\n");
    serial.write_str("  RINO Is Not an OS\n");
    serial.write_str("  Booted successfully.\n");
    serial.write_str("=============================\n");
    serial.write_str("\n");
    serial.write_str("kernel-core: arch-independent entry point\n");

    // TODO: Future subsystems will be initialized here, gated by features:
    //
    // #[cfg(feature = "allocator")]
    // allocator::init(&mut platform);
    //
    // #[cfg(feature = "scheduler")]
    // scheduler::init(&mut platform);
    //
    // #[cfg(feature = "networking")]
    // net::init(&mut platform);

    serial.write_str("No work to do. Halting.\n");
    P::halt()
}
