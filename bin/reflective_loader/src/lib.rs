use std::os::raw::c_void;
use windows_sys::Win32::Foundation::HINSTANCE;

type DWORD = i32;
type LPVOID = *mut c_void;
type BOOL = i32;
const TRUE: i32 = 1;

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(
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