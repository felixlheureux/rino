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

use core::cell::UnsafeCell;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

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
    crate::println!("=== EXCEPTION: BREAKPOINT ===");
    crate::println!("{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    crate::println!("=== EXCEPTION: DOUBLE FAULT ===");
    crate::println!("{:#?}", stack_frame);
    crate::println!("System halted.");
    loop {
        unsafe { core::arch::asm!("hlt", options(nomem, nostack)) };
    }
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let cr2 = x86_64::registers::control::Cr2::read_raw();
    crate::println!("=== EXCEPTION: PAGE FAULT ===");
    crate::println!("  Accessed address: {:#x}", cr2);
    crate::println!("  Error code: {:?}", error_code);
    crate::println!("{:#?}", stack_frame);
    crate::println!("System halted.");
    loop {
        unsafe { core::arch::asm!("hlt", options(nomem, nostack)) };
    }
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    crate::println!("=== EXCEPTION: GENERAL PROTECTION FAULT ===");
    crate::println!("  Error code: {:#x}", error_code);
    crate::println!("{:#?}", stack_frame);
    crate::println!("System halted.");
    loop {
        unsafe { core::arch::asm!("hlt", options(nomem, nostack)) };
    }
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    crate::println!("=== EXCEPTION: INVALID OPCODE ===");
    crate::println!("{:#?}", stack_frame);
    crate::println!("System halted.");
    loop {
        unsafe { core::arch::asm!("hlt", options(nomem, nostack)) };
    }
}

extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    crate::println!("=== EXCEPTION: DIVIDE BY ZERO ===");
    crate::println!("{:#?}", stack_frame);
    crate::println!("System halted.");
    loop {
        unsafe { core::arch::asm!("hlt", options(nomem, nostack)) };
    }
}
