#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rustos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use x86_64::instructions::hlt;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rustos::test_panic_handler(info)
}
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    rustos::hlt_loop();
}
// fn test_runner(tests: &[&dyn Fn()]){
//     unimplemented!();
// }
