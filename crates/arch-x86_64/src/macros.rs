//! `print!` and `println!` macros for RINO.
//!
//! These work exactly like the standard library versions, but write
//! to the serial port instead of stdout.

/// Print to the serial console (no newline).
///
/// Usage: `print!("hello")` or `print!("x = {}", 42)`
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*))
    };
}

/// Print to the serial console with a trailing newline.
///
/// Usage: `println!("hello")` or `println!("x = {}", 42)`
#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!("{}\n", format_args!($($arg)*)))
    };
}
