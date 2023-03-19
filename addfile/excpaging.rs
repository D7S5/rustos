let addr: usize = [...];

let r = 0o777;
let sign = 0o177777 << 48;


let l4_idx = (addr >> 39) & 0o777;
let l3_idx = (addr >> 30) & 0o777;
let l2_idx = (addr >> 21) & 0o777;
let l1_idx = (addr >> 21) & 0o777;

let page_offset = addr & 0o7777;

let level_4_table_addr =
    sign | ( r << 39 ) | (r << 30) | (r << 21 ) | (r << 12);

let level_3_table_addr = 
    sign | (r << 39) | ( r << 30) | ( r << 21 ) | (l4_idx << 12);

let level_2_table_addr =
    sign | (r << 39) | ( r << 30) | ( l4_idx << 21) | ( l3_idx << 12);

let level_1_table_addr =
    sign | (r << 39) | ( l4_idx << 30 ) | ( l3_idx << 21) | (l2_idx << 12);

    
let level_4_table_addr = [...];
let level_v_table_ptr = level_4_table_addr as *mut PageTable;

let recursive_page_table = unsafe {
    let level_4_table = &mut *level_4_table_ptr;
    RecursivePageTable::new(level_4_table).unwrap()
}