use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;

//vga buffer 대신 직렬 인터페이스로 인쇄
#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        SERIAL1
        .lock()
        .write_fmt(args)
        .expect("Printing to serial failed");    
    });
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}

lazy_static! { //lazy static 이용하여 사용시 정확히 한번만 호출
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { 
            SerialPort::new(0x3F8) };
            serial_port.init();
            Mutex::new(serial_port)
    };
}