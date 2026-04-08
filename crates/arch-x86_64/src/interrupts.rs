//! x86_64 interrupt handling — IDT setup and exception handlers.
//!
//! This module:
//! 1. Defines the IDT with handler functions for CPU exceptions
//! 2. Loads the IDT into the CPU via `lidt`
//! 3. Provides handler functions that print diagnostic info over serial
//!
//! All of this is x86_64-specific. Other architectures have completely
//! different interrupt mechanisms (ARM has a vector table, RISC-V has
//! CSRs for trap handling, etc.)

use crate::serial_print;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

use core::cell::UnsafeCell;

struct IdtStorage(UnsafeCell<InterruptDescriptorTable>);
unsafe impl Sync for IdtStorage {}

static IDT: IdtStorage = IdtStorage(UnsafeCell::new(InterruptDescriptorTable::new()));

/// Initialize the IDT and load it into the CPU.
///
/// After this call, CPU exceptions will invoke our handler functions
/// instead of triple-faulting.
///
/// # Safety
/// Must be called exactly once, during early boot, before any code
/// that might trigger exceptions.
pub unsafe fn init() {
    let idt = unsafe { &mut *IDT.0.get() };

    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.double_fault.set_handler_fn(double_fault_handler);
    idt.page_fault.set_handler_fn(page_fault_handler);
    idt.general_protection_fault
        .set_handler_fn(general_protection_fault_handler);
    idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
    idt.divide_error.set_handler_fn(divide_error_handler);

    idt.load();
}

// ============================================================================
// Exception handlers
//
// These functions use the `extern "x86-interrupt"` calling convention,
// which is special: the compiler generates code that correctly handles
// the interrupt stack frame the CPU pushes, and ends with `iretq`
// instead of `ret`.
//
// The `InterruptStackFrame` parameter contains:
// - instruction_pointer: where the CPU was when the interrupt fired
// - code_segment: the CS selector
// - cpu_flags: the RFLAGS register
// - stack_pointer: the RSP at the time of interrupt
// - stack_segment: the SS selector
// ============================================================================

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    serial_print("=== EXCEPTION: BREAKPOINT ===\n");
    print_stack_frame(&stack_frame);
    // Breakpoints are recoverable — execution continues after the handler.
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    // Double faults are NOT recoverable. If we get here, something is
    // seriously wrong (an exception occurred while handling another exception).
    // We print what we can and halt forever.
    serial_print("=== EXCEPTION: DOUBLE FAULT ===\n");
    print_stack_frame(&stack_frame);
    serial_print("System halted.\n");
    loop {
        unsafe { core::arch::asm!("hlt", options(nomem, nostack)) };
    }
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    serial_print("=== EXCEPTION: PAGE FAULT ===\n");

    // The CR2 register contains the virtual address that caused the fault.
    // This is the most important piece of info for debugging page faults.
    serial_print("Accessed address: ");
    let cr2 = x86_64::registers::control::Cr2::read_raw();
    print_hex(cr2);
    serial_print("\n");

    serial_print("Error code: ");
    print_hex(error_code.bits());
    serial_print("\n");

    print_stack_frame(&stack_frame);

    serial_print("System halted.\n");
    loop {
        unsafe { core::arch::asm!("hlt", options(nomem, nostack)) };
    }
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    serial_print("=== EXCEPTION: GENERAL PROTECTION FAULT ===\n");
    serial_print("Error code: ");
    print_hex(error_code);
    serial_print("\n");
    print_stack_frame(&stack_frame);
    serial_print("System halted.\n");
    loop {
        unsafe { core::arch::asm!("hlt", options(nomem, nostack)) };
    }
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    serial_print("=== EXCEPTION: INVALID OPCODE ===\n");
    print_stack_frame(&stack_frame);
    serial_print("System halted.\n");
    loop {
        unsafe { core::arch::asm!("hlt", options(nomem, nostack)) };
    }
}

extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    serial_print("=== EXCEPTION: DIVIDE BY ZERO ===\n");
    print_stack_frame(&stack_frame);
    serial_print("System halted.\n");
    loop {
        unsafe { core::arch::asm!("hlt", options(nomem, nostack)) };
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Print the interrupt stack frame fields over serial.
fn print_stack_frame(frame: &InterruptStackFrame) {
    serial_print("  Instruction pointer: ");
    print_hex(frame.instruction_pointer.as_u64());
    serial_print("\n");

    serial_print("  Code segment: ");
    print_hex(frame.code_segment.0 as u64);
    serial_print("\n");

    serial_print("  Stack pointer: ");
    print_hex(frame.stack_pointer.as_u64());
    serial_print("\n");

    serial_print("  CPU flags: ");
    print_hex(frame.cpu_flags.bits());
    serial_print("\n");
}

/// Print a u64 as hexadecimal. We can't use `core::fmt` easily without
/// a `Write` impl, so we do it manually.
fn print_hex(val: u64) {
    serial_print("0x");
    // Print 16 hex digits (64 bits / 4 bits per digit)
    for i in (0..16).rev() {
        let nibble = (val >> (i * 4)) & 0xF;
        let c = if nibble < 10 {
            b'0' + nibble as u8
        } else {
            b'a' + (nibble - 10) as u8
        };
        unsafe { crate::outb(0x3F8, c) };
    }
}
