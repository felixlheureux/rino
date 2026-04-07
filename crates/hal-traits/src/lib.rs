//! Hardware Abstraction Layer trait definitions for RINO.
//!
//! This crate defines the interfaces that architecture-specific code must
//! implement. `kernel-core` depends ONLY on these traits — it never sees
//! any concrete hardware details.
//!
//! Adding a new architecture means implementing these traits in a new
//! `arch-*` crate. `kernel-core` doesn't change.

#![no_std]

/// A serial port that can send bytes.
///
/// Each architecture implements this differently:
/// - x86_64: writes to I/O port 0x3F8 (UART 16550)
/// - AArch64: writes to a memory-mapped UART (e.g., PL011)
/// - Cortex-M: writes to a peripheral register
///
/// `kernel-core` doesn't know or care which — it just calls `write_byte`.
pub trait SerialPort {
    /// Send a single byte out the serial port.
    fn write_byte(&mut self, byte: u8);

    /// Send a string, converting `\n` to `\r\n` for serial terminals.
    ///
    /// This default implementation works for any `SerialPort`.
    /// Architectures can override it if they have a faster bulk-write path.
    fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            if byte == b'\n' {
                self.write_byte(b'\r');
            }
            self.write_byte(byte);
        }
    }
}

/// A hardware platform that the kernel can run on.
///
/// This is the top-level abstraction. Each architecture provides one
/// implementation. The kernel is generic over `P: Platform` — it works
/// on any platform that satisfies these requirements.
///
/// The associated type `Serial` lets each platform define its own serial
/// port type without dynamic dispatch (no `dyn`, no vtable, no heap).
/// The compiler monomorphizes the kernel for each concrete platform,
/// producing direct function calls with zero overhead.
pub trait Platform {
    /// The platform's serial port type.
    type Serial: SerialPort;

    /// Access the serial console for output.
    fn serial(&mut self) -> &mut Self::Serial;

    /// Halt the CPU forever. Called when there's nothing left to do.
    ///
    /// On x86_64, this executes `hlt` in a loop.
    /// On ARM, this executes `wfe` (wait for event).
    fn halt() -> !;
}
