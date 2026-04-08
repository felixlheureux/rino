//! Architecture-independent kernel logic for RINO.
//!
//! **THE RULE**: This crate must NEVER contain any architecture-specific code.
//! No x86 instructions, no ARM registers, no RISC-V CSRs. If you need
//! hardware access, add a method to a trait in `hal-traits` and implement
//! it in the appropriate `arch-*` crate.
//!
//! Everything here works on any platform that implements `hal_traits::Platform`.

#![no_std]

// Conditionally include the `alloc` crate (provides Vec, Box, String, etc.)
// This is only available when we have a heap allocator.
#[cfg(feature = "allocator")]
extern crate alloc;

#[cfg(feature = "allocator")]
pub mod allocator;

use hal_traits::{Platform, SerialPort};

pub fn kernel_main<P: Platform>(mut platform: P) -> ! {
    platform
        .serial()
        .write_str("=============================\n");
    platform.serial().write_str("  RINO Is Not an OS\n");
    platform.serial().write_str("  Booted successfully.\n");
    platform
        .serial()
        .write_str("=============================\n");
    platform.serial().write_str("\n");
    platform
        .serial()
        .write_str("kernel-core: arch-independent entry point\n");

    platform
        .serial()
        .write_str("kernel-core: arch-independent entry point\n");

    // Test: trigger a breakpoint exception.
    // If interrupts are set up correctly, this prints a message and continues.
    // If not, the CPU triple-faults and QEMU reboots.
    platform
        .serial()
        .write_str("Testing breakpoint exception...\n");
    unsafe { core::arch::asm!("int3", options(nomem, nostack)) };
    platform
        .serial()
        .write_str("Breakpoint handled! Execution continued.\n");

    #[cfg(feature = "allocator")]
    {
        if let Some((heap_start, heap_size)) = platform.heap_region() {
            allocator::init(heap_start, heap_size);
            platform
                .serial()
                .write_str("[allocator] heap initialized\n");

            {
                let mut v = alloc::vec![1, 2, 3, 4, 5];
                v.push(6);
                platform
                    .serial()
                    .write_str("[allocator] Vec allocated and used successfully\n");
            }
        } else {
            platform
                .serial()
                .write_str("[allocator] no heap region available\n");
        }
    }

    #[cfg(not(feature = "allocator"))]
    {
        platform
            .serial()
            .write_str("[no allocator] minimal build, heap disabled\n");
    }

    platform.serial().write_str("\nNo work to do. Halting.\n");
    P::halt()
}
