#![cfg(windows)]
use std::env;
use std::{fs::File, io::Write};
use winapi::shared::minwindef;
use winapi::shared::minwindef::{BOOL, DWORD, HINSTANCE, LPVOID};
use winapi::um::consoleapi;

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(
    dll_module: HINSTANCE,
    call_reason: DWORD,
    reserved: LPVOID)
    -> BOOL
{
    const DLL_PROCESS_ATTACH: DWORD = 1;
    const DLL_PROCESS_DETACH: DWORD = 0;

    match call_reason {
        DLL_PROCESS_ATTACH => demo_init(),
        DLL_PROCESS_DETACH => (),
        _ => ()
    }
    minwindef::TRUE
}

#[no_mangle]
pub fn demo_init() {
    unsafe { consoleapi::AllocConsole() };
    for (key, value) in env::vars_os() {
        // println!("{key:?}: {value:?}");
        if key == "LIBTESTFILE" {
            let mut file = File::create(value).unwrap();
            let _ = file.write_all(b"Hello, world!");
            break;
        }
    }
}

