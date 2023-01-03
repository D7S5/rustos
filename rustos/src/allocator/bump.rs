pub struct BumpAllocator {

    heap_start: usize,
    heap_end : usize,
    next : usize,
    allocations : usize,
}
//init value 0 ,
impl BumpAllocator {
    pub const fn new() -> Self {
        BumpAllocator { 
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
         }
    }
    /*heap_startheap memory, 추적
     memoery range is unused. Also, This method must be called only once
    항상 bumped 만큼 증가, 동일한 메모리 영역 두번 반환 x
    */
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}
// GlobalAlloc 정적변수 변경불가 -> 할당자 스핀록 래핑 내부변경 가능 spin::Mutex
// 다른 크레이트 정의 유형 특성구현 허용 x -> spin::Mutex 제네릭 wrap 생성 
//purpose wrapping alllcations type get mut ref

use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr;
use super::{align_up, Locked};
unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut bump = self.lock(); // get a mutable reference

        let alloc_start = align_up(bump.next, layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };
        if alloc_end > bump.heap_end {
            ptr::null_mut() // 경계
        } else {
            bump.next = alloc_end;
            bump.allocations += 1;
            alloc_start as *mut u8
        }
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let mut bump = self.lock(); // 락 이후 allocations -1

        bump.allocations -= 1;
        if bump.allocations == 0 {
            bump.next = bump.heap_start;
        }
    }
}

//GlobalAlloc Structure
/*
pub unsafe trait GlobalAlloc{
    unsafe fn alloc(&self, layout : Layout) -> *mut u8;
    unsafe fn dealloc(&self, ptr: *mut u8, layout : Layout);
    
    unsafe fn alloc_zeroed(&self, layout : Layout) -> *mut u8 {...}
    unsafe fn realloc(
        &self,
        ptr: *mut u8;
        layout : Layout,
        new_size : usize
    ) -> *mut u8 { ... }
}
 */