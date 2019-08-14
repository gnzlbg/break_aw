#![no_std]
#![no_main]

#[no_mangle]
pub unsafe extern "C" fn main() -> ! {
    let x = break_aw::foo();
    if x == 100 {
        exit(0)
    } else {
        exit(1)
    }
}

#[panic_handler]
fn panic_h(_: &core::panic::PanicInfo) -> ! { unsafe { core::hint::unreachable_unchecked() } }

#[link(name = "System", kind = "dylib")]
extern "C" {
    pub fn exit(x: i32) -> !;
}
