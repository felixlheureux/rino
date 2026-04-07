#![no_std]
#![no_main]

use bootloader_api::BootInfo;

// ============================================================================
// Serial port output
// ============================================================================

const SERIAL_PORT: u16 = 0x3F8;

/// Write one byte to an x86 I/O port.
///
/// # Safety
/// Writing to an arbitrary I/O port can have unpredictable hardware side effects.
/// The caller must ensure the port address is valid.
unsafe fn outb(port: u16, val: u8) {
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") port,
            in("al") val,
            options(nomem, nostack)
        );
    }
}

/// Write a string to the serial port, byte by byte.
/// This is our bare-metal print function.
fn serial_print(s: &str) {
    for byte in s.bytes() {
        unsafe { outb(SERIAL_PORT, byte) };
    }
}

// ============================================================================
// Kernel entry point
// ============================================================================

/// The actual kernel entry point. Called by the bootloader after it sets up
/// long mode, a stack, and basic memory mappings.
///
/// `boot_info` contains information the bootloader passes to us:
/// memory map, framebuffer details, RSDP address, etc.
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    serial_print("=============================\n");
    serial_print("  RINO Is Not an OS\n");
    serial_print("  Booted successfully.\n");
    serial_print("=============================\n");

    // We can now access boot_info for memory map, etc.
    // For now, just confirm we got it.
    serial_print("Boot info received from bootloader.\n");

    // Halt
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack));
        }
    }
}

// This macro does three things:
// 1. Creates the actual `_start` entry point symbol
// 2. Validates that your function has the correct signature
// 3. Embeds metadata so the bootloader can find and call your function
bootloader_api::entry_point!(kernel_main);

// ============================================================================
// Panic handler
// ============================================================================

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    serial_print("!!! KERNEL PANIC !!!\n");
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack));
        }
    }
}
