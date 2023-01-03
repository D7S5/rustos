#![feature(custom_test_frameworks)]
#![test_runner(rustos::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#![feature(lang_items)] //예외처리

// use core::intrinsics;
use core::panic::PanicInfo;
// use rustos::task::{simple_executor::SimpleExecutor,Task};
use rustos::task::{Task};
// use x86_64::{instructions::{hlt}, structures::paging::frame};
// use x86_64::{instructions::{hlt}, structures::paging::frame};


// #[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    rustos::hlt_loop();
}

// #[cfg(test)]
// #[panic_handler]
// #[no_mangle]
// pub fn panic(info: &PanicInfo) -> !{
//     let mut cursor : Cursor = Cursor {
//         position: 0,
//         foreground: Color::BrightRed, 
//         background:Color::BrightGreen
//      };
//     for _ in 0..(80*25) {
//         cursor.print(b"");
//     }
//     cursor.position = 160;
//     write!(cursor, "{}", info).unwrap();

//     serial_print!("[failed]\n");
//     serial_println!("Error: {}\n", info);
//     exit_qemu(QemuExitCode::Failed);
//     rustos::hlt_loop();
// }
// #[allow(unused)]
// #[derive(Clone,Copy)]
// #[repr(u8)]
// enum Color {
//     Black = 0x0,    White = 0xF,
//     Blue = 0x1,     BrightBlue = 0x9,
//     Green = 0x2,    BrightGreen = 0xA,
//     Cyan = 0x3,     BrightCyan = 0xB,
//     Red = 0x4,      BrightRed = 0xC,
//     Magenta = 0x5,  BrightMagenta = 0xD,
//     Brown = 0x6,    Yellow = 0xE,
//     Gray = 0x7,     DarkGray = 0x8
// }

// struct Cursor {
//     position : isize,
//     foreground : Color,
//     background : Color,
// }

// use core::fmt;
// use core::fmt::Write;
// impl Cursor {
//     fn color(&self) -> u8 {
//         let fg = self.foreground as u8;
//         let bg = (self.background as u8) >> 4;
//         fg | bg
//     }
//     fn print(&mut self, text: &[u8]) {
//         let color = self.color();
//         let framebuffer = 0xb8000 as *mut u8;
//         for &character in text{
//             unsafe {
//                 framebuffer.offset(self.position).write_volatile(character);
//                 framebuffer.offset(self.position + 1).write_volatile(color);
//             }
//             self.position += 2;
//         }
//     }
// }
// impl fmt::Write for Cursor{
//     fn write_str(&mut self, s: &str) -> fmt::Result {
//         self.print(s.as_bytes());
//         Ok(())
//     }
// }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode { //종료상태 지정
    Success = 0x10,
    Failed = 0x11,
}
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32); //포트 생성 이후 포트에 종료코드 write
    }
}
#[cfg(test)]
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run(); //trait >> run 
    }
    exit_qemu(QemuExitCode::Success);
}
pub trait Testable { //testcase print Trait
    fn run(&self) -> ();
}
impl<T> Testable for T 
where 
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>()); //컴파일러 직접 구현 모든 유형의 문자열 설명 반환
        self();
        serial_println!("[ok]");
    }
}
#[test_case]
fn trivial_assertion() {
    //Trait run() 구현 이후 프린트문 삭제
    //serial_print!("trivial assertion... ");
    assert_eq!(1,1);
    //println!("[ok]");
}
#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn eh_personality() {}

extern crate alloc;

//heap alloc
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};
use bootloader::{BootInfo, entry_point};
use rustos::task::keyboard;
use rustos::task::executor::Executor;
entry_point!(kernel_main);
#[no_mangle]
fn kernel_main(boot_info: &'static BootInfo) -> !  {
    use rustos::allocator;
    use rustos::memory;
    use x86_64::{structures::paging::Page, VirtAddr};
    use rustos::memory::BootInfoFrameAllocator;
    rustos::init(); //중단점 처리기 예외발생시 명령어/스택포인트 알림
    
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    let page = Page::containing_address(VirtAddr::new(0));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);
    // Dynamic heap allocate
    allocator::init_heap(&mut mapper, &mut frame_allocator)
    .expect("heap initialization failed");

    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);
    //dynamic size vector
    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    let reference_counted = Rc::new(vec![1,2,3]);
    let cloned_reference = reference_counted.clone();
    println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    core::mem::drop(reference_counted);
    println!("reference count is {} now", Rc::strong_count(&cloned_reference));

    let page_ptr : *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)};


    //keyboard executor impl async
    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();

    async fn async_number() -> u32{ 
        42
    }
    async fn example_task() {
        let number = async_number().await;
        println!("async number : {} ", number);
    }
    
    // let (level_4_page_table, _) = Cr3::read();
    // println!("Level 4 page table at : {:?}", level_4_page_table.start_address());
    
    // let ptr = 0x205a0c as *mut u32;
    // unsafe { let x = *ptr; }
    // println!("read worked");
    // unsafe { *ptr = 42; }
    // println!("read worked");

    // rustos::hlt_loop();

    // loop{
    //     use rustos::print;
    //     print!("-");
    // }
    //breakpoint exception
    // x86_64::instructions::interrupts::int3();
// 
// 무한루프 페이지 오류 처리기 미등록시 Double Falut 발생
    // unsafe{
    //     *(0xdeadbeef as *mut u64) = 42;
    // }
    // fn stack_overflow() {
    //     stack_overflow();
    // }
    // stack_overflow();
    // #[cfg(test)]
    // test_main();

    // println!("i did not crash");
    // let text = b"Snoopy's Rust OS";
    // vga_buffer::WRITER.lock().write_str("WRITER_LOCK").unwrap();
    // write!(vga_buffer::WRITER.lock(), ", Some numbers: {} {} ", 7, 1.337).unwrap();

    // let mut curser = Cursor {
    //     position : 0,
    //     foreground: Color::BrightCyan,
    //     background: Color::Black,
    // };
    // curser.print(text);
   
    // loop {}
}
mod vga_buffer;
mod serial;