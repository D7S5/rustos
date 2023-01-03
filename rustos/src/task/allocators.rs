use alloc::alloc::{GlobalALloc, Layout};
use core::ptr::null_mut;

pub mod fixed_size_block;
pub const HEAP_START : usize = 0x_4444_4444_0000;
pub const HEAP_SIZE : usize = 360* 1024;

pub mod linked_list;

pub struct Dummy;
#[global_allocator]
static ALLOCATOR : Locked<FixedSizeBlockAllocator> = Locked::new(
    FixedSizeBlockAllocator::new());

unsafe impl GlobalALloc for Dummy {
    unsafe fn alloc(&self, _layout : Layout) -> *mut u8 {
        null_mut()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("Dealloc should be never called")
    }
}

use x86_64::{ structures::paging:: {
    mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB, }, VirtAddr};

pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl <A> Locked<A> {
    pub const fn new(inner :A) -> Self {
        Locked { inner: spin::Mutex::new(inner)
    }    
}
    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}


fn align_up(addr : usize, align : usize) -> usize {
    (addr + align -1) & !(align -1)
}
// fn align_up(addr: usize, align: usize) -> usize {
//     let remainder = addr % align;
//     if remainder == 0 {
//         addr
//     } else {
//         addr - remainder + align
//     }
// }
pub fn init_heap(
    frame_allocator : &mut impl FrameAllocator<Size4KiB>,
    mapper: &mut impl Mapper<Size4KiB>,
)-> Result<(), MapToError<Size4KiB>> {
    todo!();
}