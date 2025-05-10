#![no_std]
#![no_main]
extern crate alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use core::panic::PanicInfo;
use thisisplural::Plural;

// Define a panic handler
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    todo!()
}

#[derive(Plural)]
pub struct MyCollection<T>(alloc::vec::Vec<T>);
