#![no_std]
#![no_main]

use core::ffi::c_char;
use cortex_m_rt::entry;
use panic_halt as _;

extern "C" {
    fn bsp_init();
    fn print_Line(line: i32, text: *const c_char);
}

#[entry]
fn main() -> ! {
    unsafe { bsp_init(); }

    static HELLO: &[u8] = b"Hello from Rust!\0";
    unsafe {
        // line 0 on the board's LCD
        print_Line(0, HELLO.as_ptr() as *const c_char);
    }

    loop {
        // spin forever
        cortex_m::asm::nop();
    }
}
