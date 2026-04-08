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

/// Helper to reduce verbosity of serial printing.
fn kprint<P: Platform>(platform: &mut P, s: &str) {
    platform.serial().write_str(s);
}

pub fn kernel_main<P: Platform>(mut platform: P) -> ! {
    kprint(&mut platform, "=============================\n");
    kprint(&mut platform, "  RINO Is Not an OS\n");
    kprint(&mut platform, "  Booted successfully.\n");
    kprint(&mut platform, "=============================\n");
    kprint(&mut platform, "\n");
    kprint(&mut platform, "kernel-core: arch-independent entry point\n");

    kprint(&mut platform, "Testing breakpoint exception...\n");
    unsafe { core::arch::asm!("int3", options(nomem, nostack)) };
    kprint(&mut platform, "Breakpoint handled! Execution continued.\n");

    #[cfg(feature = "allocator")]
    {
        if let Some((heap_start, heap_size)) = platform.heap_region() {
            allocator::init(heap_start, heap_size);
            kprint(&mut platform, "[allocator] heap initialized\n");

            {
                let mut v = alloc::vec![1, 2, 3, 4, 5];
                v.push(6);
                kprint(
                    &mut platform,
                    "[allocator] Vec allocated and used successfully\n",
                );
            }
        } else {
            kprint(&mut platform, "[allocator] no heap region available\n");
        }
    }

    #[cfg(not(feature = "allocator"))]
    {
        kprint(
            &mut platform,
            "[no allocator] minimal build, heap disabled\n",
        );
    }

    kprint(&mut platform, "\nNo work to do. Halting.\n");
    P::halt()
}

#[cfg(feature = "tests")]
pub fn run_tests<P: Platform>(platform: &mut P) {
    platform.serial().write_str("[test] breakpoint...");
    unsafe { core::arch::asm!("int3", options(nomem, nostack)) };
    platform.serial().write_str(" OK\n");

    platform.serial().write_str("[test] heap alloc...");
    let v = alloc::vec![1, 2, 3];
    assert!(v.len() == 3);
    platform.serial().write_str(" OK\n");
}
