//! x86_64 platform implementation for RINO.
//!
//! This crate contains ALL x86_64-specific code:
//! - UART 16550 serial driver (I/O port access)
//! - CPU halt instruction
//! - (later) interrupt descriptor table, page tables, APIC, etc.
//!
//! All `unsafe` hardware interaction for x86 is confined here.

#![no_std]
#![feature(abi_x86_interrupt)]

pub mod interrupts;
pub mod serial;

#[macro_use]
pub mod macros;

use hal_traits::{Platform, SerialPort};

// ============================================================================
// UART 16550 serial driver
// ============================================================================

/// I/O port address of COM1 (first serial port).
const COM1: u16 = 0x3F8;

/// A UART 16550 serial port accessed via x86 I/O ports.
///
/// This is the standard serial controller found in every PC since 1981.
/// QEMU emulates one at port 0x3F8.
pub struct Uart16550 {
    port: u16,
}

impl Uart16550 {
    /// Create a new UART driver for the given I/O port base address.
    ///
    /// # Safety
    /// The caller must ensure:
    /// - The port address is valid and has a UART behind it
    /// - This is called only once per port (no aliasing)
    /// - We're running in ring 0 (kernel mode) so `out`/`in` won't fault
    pub unsafe fn new(port: u16) -> Self {
        // Initialize the UART:
        // In a full driver, you'd set baud rate, data bits, FIFO, etc.
        // QEMU's emulated UART works without initialization, but real
        // hardware needs this. We'll add proper init later.
        Self { port }
    }
}

impl SerialPort for Uart16550 {
    fn write_byte(&mut self, byte: u8) {
        // In a proper driver, we'd poll the Line Status Register (port + 5)
        // to check if the transmit buffer is empty before writing.
        // QEMU's UART is always ready, so we skip that for now.
        unsafe { outb(self.port, byte) };
    }
}

// ============================================================================
// x86_64 Platform
// ============================================================================

/// The x86_64 platform, implementing `hal_traits::Platform`.
///
/// This struct owns all the hardware resources for x86_64.
/// Creating it initializes the hardware (serial port, and later
/// interrupts, memory, etc.)
pub struct X86_64Platform {
    serial: Uart16550,
}

impl X86_64Platform {
    /// Initialize the x86_64 platform.
    ///
    /// # Safety
    /// Must be called exactly once, early in the boot process.
    pub unsafe fn init() -> Self {
        let serial = unsafe { Uart16550::new(COM1) };

        // Set up interrupt handlers before anything can fault
        unsafe { interrupts::init() };

        Self { serial }
    }
}

use core::cell::UnsafeCell;

struct Heap(UnsafeCell<[u8; 64 * 1024]>);

unsafe impl Sync for Heap {}

static HEAP: Heap = Heap(UnsafeCell::new([0; 64 * 1024]));

impl Platform for X86_64Platform {
    type Serial = Uart16550;

    fn serial(&mut self) -> &mut Self::Serial {
        &mut self.serial
    }

    fn heap_region(&self) -> Option<(*mut u8, usize)> {
        Some((HEAP.0.get() as *mut u8, 64 * 1024))
    }

    fn halt() -> ! {
        loop {
            unsafe {
                core::arch::asm!("hlt", options(nomem, nostack));
            }
        }
    }
}

// ============================================================================
// Raw x86 I/O port access
// ============================================================================

/// Write a single byte to an x86 I/O port.
///
/// # Safety
/// Writing to an arbitrary I/O port can have unpredictable hardware effects.
#[inline]
pub(crate) unsafe fn outb(port: u16, val: u8) {
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") port,
            in("al") val,
            options(nomem, nostack)
        );
    }
}

#[inline]
pub(crate) unsafe fn inb(port: u16) -> u8 {
    let val: u8;
    unsafe {
        core::arch::asm!(
            "in al, dx",
            out("al") val,
            in("dx") port,
            options(nomem, nostack)
        );
    }
    val
}
