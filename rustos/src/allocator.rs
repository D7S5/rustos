use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

pub struct Dummy;
pub mod fixed_size_block;
pub const HEAP_START : usize = 0x_4444_4444_0000;
pub const HEAP_SIZE : usize = 100 * 1024;

pub mod linked_list;
pub mod bump;
// use bump::BumpAllocator;
// use linked_list::LinkedListAllocator;
use fixed_size_block::FixedSizeBlockAllocator;
//define global_allocator , BumpAllocator, LinkedListAllocator, FixedSizedBlockAlloctor
#[global_allocator]
static ALLOCATOR : Locked<FixedSizeBlockAllocator> = Locked::new(
    FixedSizeBlockAllocator::new());

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout : Layout) {
        panic!("Dealloc should be never called")
    }
}
use x86_64::{ structures::paging::{
    mapper::MapToError , FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
}, VirtAddr
};
pub struct Locked<A> {
    inner: spin::Mutex<A>,
}
impl<A> Locked<A> {
    pub const fn new(inner: A ) -> Self {
        Locked { inner: spin::Mutex::new(inner) 
        }
    }
    pub fn lock(&self) -> spin::MutexGuard<A> { //lock return Guard Type
        self.inner.lock()
    }
}
// fn align_up(addr: usize, align: usize) -> usize { //commonly align_up
//     let remainder = addr % align;
//     if remainder == 0 {
//         addr  //addr already aligned
//     } else {    
//         addr- remainder + align
//     }
// }
//opt only 2의 sqr GlobalAlloc 보장 address aligned create bitmask 이후 effective align
//align -up optimized bitmask // efficient way
fn align_up(addr: usize, align : usize) -> usize {
    (addr + align - 1) & !(align -1 )
}
// heap init 
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
)-> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        //pub const HEAP_START : usize = 0x_4444_4444_0000;
        // pub const HEAP_SIZE : usize = 100 * 1024;
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end =  heap_start + HEAP_SIZE - 1u64; //base address + HEAP_SIZE = HEAP_END
        
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end); 
        //allocate heap segment 
        Page::range_inclusive(heap_start_page, heap_end_page)  
    };

    for page in page_range {
        let frame = frame_allocator
        .allocate_frame()
        .ok_or(MapToError::FrameAllocationFailed)?;
        
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        //set page flag rwx
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
            ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);   //heap 할당 범위 락. init
        };
    }
Ok(())
}