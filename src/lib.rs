#![feature(abi_x86_interrupt)]
#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

pub mod gdt;
pub mod interrupts;
pub mod serial;
pub mod vga_buffer;

// Test Running & Formatting
//////////////////////////////////////////////////////////////////////////

use core::fmt;

// TODO: Move into crate for TERMOUT/ or at least mod?
pub struct Green(pub &'static str);
pub struct Red(pub &'static str);

impl fmt::Display for Green {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\x1B[32m")?; // prefix code
        write!(f, "{}", self.0)?;
        write!(f, "\x1B[0m")?; // postfix code
        Ok(())
    }
}
impl fmt::Display for Red {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\x1B[31m")?; // prefix code
        write!(f, "{}", self.0)?;
        write!(f, "\x1B[0m")?; // postfix code
        Ok(())
    }
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("{}", Green("[ok]"));
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("{} {}\n", Red("Error:"), info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

// Init
// //////////

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    interrupts::init_pic();
    x86_64::instructions::interrupts::enable(); // should this move into interrupts?
}

// Test Entry Point
// //////////
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    hlt_loop();
}

// HLT Loop
/////////////

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

// Panic
// //////////
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
// Exit
// //////////

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}
