//! Global serial port writer for RINO.
//!
//! Provides a locked, globally-accessible serial port that implements
//! `core::fmt::Write`. This enables `print!` and `println!` macros
//! that work from anywhere in the kernel — including interrupt handlers
//! and the panic handler.

use crate::{inb, outb};
use core::fmt;
use core::fmt::Write;
use core::sync::atomic::{AtomicBool, Ordering};

/// The I/O port base address of COM1.
const COM1: u16 = 0x3F8;

/// A minimal spinlock protecting the serial port.
///
/// On a single-core system (which we are right now), this primarily
/// prevents reentrant access — e.g., if an interrupt fires while
/// we're in the middle of printing.
///
/// On multi-core, this would need to disable interrupts while held
/// to prevent deadlock. We'll add that when we support SMP.
struct SpinLock {
    locked: AtomicBool,
}

impl SpinLock {
    const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
        }
    }

    fn lock(&self) {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            // Spin. On x86, `pause` hints to the CPU that we're in a spin
            // loop, improving performance on hyperthreaded cores.
            core::hint::spin_loop();
        }
    }

    fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

/// The global serial writer and its protecting lock.
static SERIAL_LOCK: SpinLock = SpinLock::new();

/// A writer that sends bytes to COM1 via x86 I/O ports.
///
/// This struct implements `core::fmt::Write`, which means it can be
/// used with `write!` and `writeln!` macros for formatted output.
pub struct SerialWriter;

impl SerialWriter {
    /// Write a single byte to COM1.
    #[inline]
    fn write_byte(&mut self, byte: u8) {
        // Wait for transmit buffer to be empty.
        // Bit 5 of the Line Status Register (port + 5) indicates
        // the transmit holding register is empty.
        while (unsafe { inb(COM1 + 5) } & 0x20) == 0 {
            core::hint::spin_loop();
        }
        unsafe { outb(COM1, byte) };
    }
}

impl fmt::Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            if byte == b'\n' {
                self.write_byte(b'\r');
            }
            self.write_byte(byte);
        }
        Ok(())
    }
}

/// Print to the serial console, with the lock held.
///
/// This is the function that `print!` and `println!` call.
/// It acquires the spinlock, writes the formatted string, and releases.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    SERIAL_LOCK.lock();
    let mut writer = SerialWriter;
    writer.write_fmt(args).unwrap();
    SERIAL_LOCK.unlock();
}
