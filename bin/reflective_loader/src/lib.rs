#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

use core::ffi::c_void;
use windows_sys::Win32::Foundation::HINSTANCE;

mod loader; 

type DWORD = i32;
type LPVOID = *mut c_void;
type BOOL = i32;
const TRUE: i32 = 1;

// #[allow(non_upper_case_globals)]
// #[export_name = "_fltused"]
// pub static _fltused: i32 = 0;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! { loop {} }

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub unsafe extern "system" fn _DllMainCRTStartup(
    dll_module: HINSTANCE,
    call_reason: DWORD,
    reserved: LPVOID)
    -> BOOL
{
    match call_reason {
        _ => ()
    }
    TRUE
}