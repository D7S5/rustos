// use x86_64::structures::paging::{Mapper, Page, PageTable, RecursivePageTable};
// use x86_64::{VirtAddr , PhysAddr};

/*Page	0o_SSSSSS_AAA_BBB_CCC_DDD_EEEE
Level 1 Table Entry	0o_SSSSSS_RRR_AAA_BBB_CCC_DDDD
Level 2 Table Entry	0o_SSSSSS_RRR_RRR_AAA_BBB_CCCC
Level 3 Table Entry	0o_SSSSSS_RRR_RRR_RRR_AAA_BBBB
Level 4 Table Entry	0o_SSSSSS_RRR_RRR_RRR_RRR_AAAA */


// code is virtual memory 512GiB 48-bit address space
let addr: usize : [...];

let r = 0o777; //recursive index
let sign = 0o177777 << 48; 

//shifing operation 
let l4_idx = (addr >> 39) & 0o777; // level 4 index
let l3_idx = (addr >> 30) & 0o777; // level 3 index
let l2_idx = (addr >> 21) & 0o777; // level 2 index
let l1_idx = (addr >> 21) & 0o777; // level 1 index



//struct page level  : offset 12bit + 9bit + 9bit + 9bit + 9bit + 16bit sign extension  == 64bit
let page_offset = addr & 0o7777; // 

let level_4_table_addr = 
    sign | (r << 39) | (r << 30) | (r << 21) | (r << 12);

let level_3_table_addr =
    sign | (r << 39) | (r << 30) | (r << 21) | (l4_idx << 12);

let level_2_table_addr =
    sign | (r << 39) | (r << 30) | (l4_idx << 21) | (l3_idx << 12);

let level_1_table_addr =
    sign | (r << 39) | (l4_idx << 30) | (l3_idx << 21) | (l2_idx << 12);
    
    
let level_4_table_addr = [...];
let level_4_table_ptr = level_4_table_addr as *mut PageTable;
let recursive_page_table = unsafe{
     let level_4_table = &mut *level_4_table_ptr;
     RecursivePageTable::new(level_4_table).unwrap();
}

let addr: u64 = [...];
let addr = VirtAddr::new(addr);

let page : Page = Page:containing_address(addr);

let frame = recursive_page_table.translate_page(page);
frame.map(|frame| frame.start_address() + u64::from(addr.page.offset()))
