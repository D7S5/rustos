
use x86_64::structures::paging::OffsetPageTable;
use x86_64:: {VirtAddr,
    structures::paging::{Page, FrameAllocator, Size4KiB, Mapper, PhysFrame, PageTable}
};

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}
impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map : &'static MemoryMap) -> Self {
        BootInfoFrameAllocator { 
            memory_map, // 메모리 맵의 참조
            next: 0, //다음 프레임 번호 추적 필드
        }
    }
    fn usable_frame(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions
        .filter(|r| r.region_type == MemoryRegionType::Usable);

        let addr_ranges = usable_regions
        .map(|r| r.range.start_addr()..r.range.end_addr());

        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}
unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frame().nth(self.next);
        self.next += 1;
        frame
    }
}
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}
//mapping 기능 테스트
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}
pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}
//make private
//가상 주소 물리적 주소 변환 OS 커널의 일반적인 작업
//lv4 page table 참조 반환 함수 생성  init 함수 생성 이후 비공개 함수로 변경
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr)
-> &'static mut PageTable{
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read(); //물리적 프레임 read

    let phys = level_4_table_frame.start_address(); //물리적 시작주소 변환
    let virt = physical_memory_offset + phys.as_u64();//페이지 테이블 프레임 매핑 가상주소 get
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();//원시포인터로 변환

    &mut *page_table_ptr //unsafe
}

//가상주소 물리적 주소로 변환, 4단계 페이지 통과 필요
use x86_64::PhysAddr;

pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr>
{
    translate_addr_inner(addr, physical_memory_offset)
}

fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr)
-> Option<PhysAddr> {
    use x86_64::structures::paging::page_table::FrameError;
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();
    
    let table_indexes = [
        addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()];
    let mut frame = level_4_table_frame;

    for &index in &table_indexes {
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe {&*table_ptr};
        
        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
        };
    }
    Some(frame.start_address() + u64::from(addr.page_offset()))

}
