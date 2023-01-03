// use core::iter::Scan;

use x86_64::{structures::idt::{InterruptDescriptorTable, InterruptStackFrame} };
use crate::println;

use lazy_static::lazy_static;
use crate::gdt;

use pic8259::ChainedPics;
use spin;

//offset 여유번호 32- 47 
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe {
        ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
    });

#[derive(Debug, Clone,Copy)]
#[repr(u8)]
//interrupt Index set, irq,
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

//u8 변환 rust enum C와 유사 변형 가능 define interrupt index structure 
impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }
    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}
use crate::print;
lazy_static! {
static ref IDT : InterruptDescriptorTable = {
            let mut idt = InterruptDescriptorTable::new();
            //page default handler init
            idt.breakpoint.set_handler_fn(breakpoint_handler);
            idt.double_fault.set_handler_fn(double_fault_handler);
            unsafe {
                idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
            }
            //timer interrupt handler 
            idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);
            idt[InterruptIndex::Keyboard.as_usize()]
            .set_handler_fn(keyboard_interrupt_handler);
            //memory interrupt handler
            idt.page_fault.set_handler_fn(page_fault_handler);
            idt  
};
}
//x86_64 제공 Errorcode 
use x86_64::structures::idt::PageFaultErrorCode;
use crate::hlt_loop;

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop(); //Page Fault시 진행불가 hlt
}


//timer interrupt handelr
extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: InterruptStackFrame)
    {
        print!(".");
        unsafe{
            PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
        }
    }
extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: InterruptStackFrame)
    {
        //keyboard 스캔코드 읽고 키 입력시 스캔코드 출력
        //포트 생성 이후 PS/2 컨트롤러의 데이터 포트에서 read
        //0x60 번호 존재 포트 스캔코드 누르기 0 릴리스 1 최상위 1비트
        
        use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
        use spin::Mutex;
        use x86_64::instructions::port::Port;
        //Mutex 보호 lazy_static 개체 생성
        lazy_static! {
            static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1,
            HandleControl::Ignore)
        );
        }          
        let mut keyboard = KEYBOARD.lock();

        //보통 keyboard, timer Port 관습적 고정값 사용
        let mut port = Port::new(0x60);
        //port data unsafe read
        let scancode: u8 = unsafe { port.read() };
        crate::task::keyboard::add_scancode(scancode);
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode){
            //keyboard interrupts signal
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => print!("{}", character),
                    DecodedKey::RawKey(key) => print!("{:?}", key),
                }
            }
        }
        //key 입력시 print구간 //스캔코드 부재
        unsafe {
            PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
        }
    }

//키보드 고유값 port를 통해 read, -> signal 발생(process_keyevent)-> interrupt handling 
// -> read data Decodekey:unicode or Rawkey -> print!( charset or key)

//if handelr call exception 발생시
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64) -> !
    {
        panic!("EXCEPTION: DOUBLE FALUT\n{:#?}", stack_frame);
    }
/*cpu exception Page Fault, Invalid Opcode 명령어 유효 x, 일반 보호 오류, 
Double Fault : 예외 발생 CPU 핸들러 함수 호출, 호출의 간격에 또 다른 예외 발생시
CPU 이중 오류 예외 발생, 핸들러함수 미존재시에도 발생
Triple Fault: Double Falut의 연장선, 치명적 오류 대개 운영체제 재부팅

오류테이블 종류
#[repr(C)]
pub struct InterruptDescriptorTable {
    pub divide_by_zero: Entry<HandlerFunc>,
    pub debug: Entry<HandlerFunc>,
    pub non_maskable_interrupt: Entry<HandlerFunc>,
    pub breakpoint: Entry<HandlerFunc>,
    pub overflow: Entry<HandlerFunc>,
    pub bound_range_exceeded: Entry<HandlerFunc>,
    pub invalid_opcode: Entry<HandlerFunc>,
    pub device_not_available: Entry<HandlerFunc>,
    pub double_fault: Entry<HandlerFuncWithErrCode>,
    pub invalid_tss: Entry<HandlerFuncWithErrCode>,
    pub segment_not_present: Entry<HandlerFuncWithErrCode>,
    pub stack_segment_fault: Entry<HandlerFuncWithErrCode>,
    pub general_protection_fault: Entry<HandlerFuncWithErrCode>,
    pub page_fault: Entry<PageFaultHandlerFunc>,
    pub x87_floating_point: Entry<HandlerFunc>,
    pub alignment_check: Entry<HandlerFuncWithErrCode>,
    pub machine_check: Entry<HandlerFunc>,
    pub simd_floating_point: Entry<HandlerFunc>,
    pub virtualization: Entry<HandlerFunc>,
    pub security_exception: Entry<HandlerFuncWithErrCode>,
    // some fields omitted
}

//데이터 경합 문제 UNSAFE 처리 필요 lazy_static_처리로 컴파일시 데이터 위치 지정 x unsafe 문제 회피*/
pub fn init_idt() {
    IDT.load(); //load 프로그램 전체 런타임 유효 필요성
}
//조건 #![feature(abi_x86_interrupt)]
extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame)
    {
        println!("EXCEPTION : BREAKPOINT\n{:#?}", stack_frame);
    }

#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}