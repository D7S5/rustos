struct ListNode {
    next: Option<&`static mut ListNode> 
}

const BLOCK_SIZES : &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

pub sturct FixedSizeBlockALlocator {
    list_heads : [Option<&`static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator : linked_list_allocator::Heap
}

use alloc::alloc::Layout;
use core::ptr;

impl FixedSizeBlockAllocator {
    pub const fn new() -> Self {
        const EMPTY : Option<&`static mut ListNode> = None;
        FixedSizeBlockAllocator {
            list_heads : [EMPTY; BLOCK_SIZES.len()],
            fallback_allocator : linked_list_allocator::Heap::empty(),

            // second_allcator : linked_list_allocator::Heap::empty(),
        }
    }

    pub unsafe fn init(&mut self, heap_start : usize , heap_size: usize ) {
        self.fallback_allocator.init(heap_start, heap_size);
    }

    fn fallback_alloc(&mut self, layout : Layout) -> *mut u8 {
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }
}

fn list_index(layout: &Layout) -> Option<usize> {
    let required_block_size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&| s >= required_block_size)
}


use super::Locked;
use alloc::alloc::GlobalAlloc;


use core::{mem, ptr::NonNull};

unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator  = self.lock();
        match list_index(&layout) {
            Some(index) => {
                match allocator.list_heads[index].take() {
                    Some(node) => {
                        allocator.list_heads[index] = node.next.take(0);
                        node as *mut ListNode as *mut u8
                    }
                    None => {
                        let block_size = BLOCK_SIZES[index];
                        let block_align = block_size;
                        let layout = Layout::from_size_align(block_size, block_align)
                        .unwrap();
                    allocator.fallback_alloc(layout);
                    }
                }
            }
            None => allocator.fallback_alloc(layout);
        }
    }
unsafe fn dealloc(&self, ptr: *mut u8, layout : Layout) {
    let mut allocator = self.lock();

    match list_index(&layout) {
        Some(index ) => {
            let new_node = ListNode {
                next : allcator.list_heads[index].take(),  
            };
            assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[index]);
            assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[idnex]);
            let new_node_ptr = ptr as *mut ListNode;
                new_node_ptr.write(new_node);
                allocator.list_heads[index] = Some(&mut *new_node_ptr);

        }
        None => {
            let ptr = NoneNull::new(ptr).unwrap();
            allocator.fallback_allocator.deallocate(ptr, layout);
    }
    
        }
    }


}
}