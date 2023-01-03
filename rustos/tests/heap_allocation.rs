#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rustos::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

entry_point!(main);

#[panic_handler]
fn panic(info: &PanicInfo) -> !{
    rustos::test_panic_handler(info)
}

fn main(boot_info: &'static BootInfo) -> ! {
    use rustos::allocator;
    use rustos::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    rustos::init();
    
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe {
        memory::init(phys_mem_offset) };
    
    
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    allocator::init_heap(&mut mapper, &mut frame_allocator)
    .expect("Heap initialization failed");

    test_main();
    loop {}
}

use alloc::boxed::Box;
#[test_case]
fn simple_allocation() {
        let heap_value_1 = Box::new(41);
        let heap_value_2 = Box::new(13);
        let heap_value_3 = Box::new(100);

        assert_eq!(*heap_value_1, 41);
        assert_eq!(*heap_value_2, 13);
        assert_eq!(*heap_value_3, 100);
    }
use alloc::vec::Vec;
#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1 ) * n / 2);
}
// 
#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i)
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n /2 );
}
// 
use rustos::allocator::HEAP_SIZE;
#[test_case]
fn many_boxes() {
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i );
    }
}
//재사용 문제로 인한 failed -> Solution Linked list Allocator
//bump 루프 과정 중 지속되는 메모리 할당 반복된 메모리 사용 재할당문제 발생 allocation count 1 > 2 > 1  
// count 0일 경우 할당 해제 루프문 이전 해제 x
#[test_case]
fn many_boxes_long_lived() {
    let long_lived = Box::new(1);
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x , i);
    }
    assert_eq!(*long_lived, 1);
    
}

// 격리 스레드 하나 생성 후 거기서 아토믹처리나 락 관련 연산하는 것