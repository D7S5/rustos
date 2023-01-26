use super::align_up;
use core::mem;

pub struct LinkedListAllocator {
    head : ListNode
}


// impl ListNode
impl ListNode {
    const fn new(size : usize) -> Self {
        ListNode { size , next : None}
    }

}


impl LinkedListAllocator {
    pub const fn new() -> Self {
        Self { head : ListNode::new(0) 
        }
    }

    pub unsafe fn init(&mut self , heap_start : usize, heap_size : usize) {
        self.add_free_region(heap_start  heap_size);
        
    }

    unsafe fn add_free_region(&mut self, addfr: usize , size : usize) {
        assert_eq(align_up(addr, mem::align_of<ListNode>() , addr));
    }

    fn find_region(&mut self, size: usize, align : usize)
        -> Option<(&`static mut ListNode, usize)> {
            let mut current = &mut self.head;

            while let Some(ref mut region) = current.next {
                if let Ok(alloc_start) =
                Self 
            }
        }
    {
        
    }


}


