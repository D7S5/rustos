
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)] //Copy Semantics
#[repr(u8)] //enum Size u4 >> u8 
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Megenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]  // chars filed type
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] //Struct field order == C//
struct ScreenChar {
    ascii_character : u8,
    color_code : ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH : usize = 80;

use volatile::Volatile; //최적화 과정 중 쓰기작업 불필요시 삭제 방지 Volatile 버전주의
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

//메모리 주소상 고정 정적변수 Writer 지정 전역 접근 가능 인터페이스 컴파일시 지정 
//Colornew:: error > const 로 해결 // column_position 문제는 raw pointer를 레퍼런스로 변경 불가 lazy_static 이용
use spin::Mutex; //스핀 락 구현 루프돌면서 권한 대기
use lazy_static::lazy_static;
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}
//Writer 항상 마지막 행에서 값 작성/ 작성규칙
pub struct Writer {
    column_position: usize,
    color_code : ColorCode,
    buffer : &'static mut Buffer, // Buffer lifetime 명시
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH { // 행 데이터 확인
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1 ;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }
    //출력값 순회 개행
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT -1);
        self.column_position = 0;
    }
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ', //스페이스
            color_code : self.color_code //상태값 유지 코드 클리어
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank); //모든 문자 스페이스 문자로 치환
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                //ASCII CODE SCOPE
                _ => self.write_byte(0xfe),
            }

        }
    }
}
// formatting macro int , float -> core::fmt::Write impl

use core::fmt;

impl fmt::Write for Writer {
    fn write_str(&mut self , s:&str)-> fmt::Result {
        self.write_string(s);
        Ok(()) //쓰기 이후 Ok 반환 Err시 Panic!
    }
}
//이후 write!/ writeln! 사용가능

//println! 매크로 정의
#[macro_export] //크레이트 전역 사용가능 use crate::println import
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
//Mutex lock 중 인터럽트 비활성화
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();    
    });
    
}

#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for ( i,c ) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT -2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
        });    
}

/*
#[test_case]
fn test_println_many_case() {
    for _ in 0..200 {
    println!("test_print output");}
}
#[test_case]
fn test_println_output() {
    let s = "Some test string that fits on a single line";
    println!("{}", s);
    for (i, c) in s.chars().enumerate() {
        let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT -2][i].read();
        assert_eq!(char::from(screen_char.ascii_character), c);
    }
    
}
 */


//ex print

// pub fn print_something() {
//     use core::fmt::Write;

//     let mut writer = Writer {
//         column_position: 0,
//         color_code: ColorCode::new(Color::Megenta, Color::White),
        
//         buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
//     };
//     writer.write_byte(b'H');
//     writer.write_string("ello " );
//     write!(writer, "The numbers are int {}, float {} ",7, 1.0/3.0).unwrap();
// }



//Support to text buffer ASCII, Codepage 437 