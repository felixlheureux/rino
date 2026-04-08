//! RINO kernel entry point for x86_64 BIOS boot.
//!
//! This is intentionally thin. It:
//! 1. Receives BootInfo from the bootloader
//! 2. Initializes the x86_64 platform
//! 3. Hands off to kernel_core::kernel_main
//!
//! If you're adding kernel logic, it belongs in `kernel-core`, not here.
//! If you're adding x86 hardware code, it belongs in `arch-x86_64`, not here.

#![no_std]
#![no_main]

use bootloader_api::BootInfo;
use hal_traits::Platform;

/// Entry point called by the bootloader.
fn kernel_entry(_boot_info: &'static mut BootInfo) -> ! {
    // Initialize x86_64 hardware
    let platform = unsafe { arch_x86_64::X86_64Platform::init() };

    // Hand off to the arch-independent kernel
    kernel_core::kernel_main(platform)
}

bootloader_api::entry_point!(kernel_entry);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    arch_x86_64::println!("=== KERNEL PANIC ===");
    arch_x86_64::println!("{}", info);
    loop {
        unsafe { core::arch::asm!("hlt", options(nomem, nostack)) };
    }
}
