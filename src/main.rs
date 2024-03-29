#![feature(custom_test_frameworks)]
#![feature(type_ascription)]
#![allow(deprecated)]
#![no_std]
#![no_main]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;
use blog_os::{
    allocator,
    println,
    task::{executor::Executor, keyboard, Task},
};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

// Panic Handler
/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    blog_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! { blog_os::test_panic_handler(info) }

// EntryPoint
entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello World!");
    blog_os::init();
    allocator::init_kernel_heap(boot_info);

    #[cfg(test)]
    test_main();

    println!("It did not crash!");

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
}

// Async Tryout

async fn async_number() -> u32 { 42 }

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}
