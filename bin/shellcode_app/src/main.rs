#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::arch::asm;

// Syscall numbers for Linux x86_64
const SYS_OPEN: u64 = 2;
const SYS_CLOSE: u64 = 3;
const SYS_EXIT: u64 = 60;

// File flags
const O_CREAT: i32 = 0o100;
const O_WRONLY: i32 = 0o1;
const O_TRUNC: i32 = 0o1000;

// File permissions (0644)
const S_IRUSR: u32 = 0o400;
const S_IWUSR: u32 = 0o200;
const S_IRGRP: u32 = 0o40;
const S_IROTH: u32 = 0o4;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        syscall1(SYS_EXIT, 1);
    }
    loop {}
}

// Syscall wrappers using inline assembly
#[inline(always)]
unsafe fn syscall1(n: u64, arg1: u64) -> u64 {
    let ret: u64;
    asm!(
        "syscall",
        in("rax") n,
        in("rdi") arg1,
        lateout("rax") ret,
        options(nostack)
    );
    ret
}

#[inline(always)]
unsafe fn syscall3(n: u64, arg1: u64, arg2: u64, arg3: u64) -> i64 {
    let ret: i64;
    asm!(
        "syscall",
        in("rax") n,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        lateout("rax") ret,
        options(nostack)
    );
    ret
}

// String comparison function
#[inline(always)]
unsafe fn str_starts_with(haystack: *const u8, needle: &[u8]) -> bool {
    for i in 0..needle.len() {
        if *haystack.add(i) != needle[i] {
            return false;
        }
    }
    true
}


// Environment variable to look for
const ENV_VAR_NAME: &[u8] = b"SHELLCODE_FILE_PATH=";

// Entry point - expects environment pointer in rdi
#[no_mangle]
#[link_section = ".text"]
pub unsafe extern "C" fn _start() -> ! {
    // Get environment pointer from initial stack
    // On Linux, the stack layout is:
    // argc
    // argv[0..argc]
    // NULL
    // envp[0..N]
    // NULL

    let mut env_ptr: *const *const u8;

    // Read from stack pointer
    asm!(
        "mov {}, rsp",
        out(reg) env_ptr,
    );

    // Skip argc
    env_ptr = env_ptr.add(1);

    // Skip argv (until we find NULL)
    while !(*env_ptr).is_null() {
        env_ptr = env_ptr.add(1);
    }

    // Skip the NULL after argv
    env_ptr = env_ptr.add(1);

    // Now env_ptr points to envp
    let mut file_path: *const u8 = core::ptr::null();

    // Search for our environment variable
    while !(*env_ptr).is_null() {
        let env_str = *env_ptr;
        if str_starts_with(env_str, ENV_VAR_NAME) {
            // Found it! Extract the value after the '='
            file_path = env_str.add(ENV_VAR_NAME.len());
            break;
        }
        env_ptr = env_ptr.add(1);
    }

    // If we found the environment variable, create the file
    if !file_path.is_null() {
        let flags = (O_CREAT | O_WRONLY | O_TRUNC) as u64;
        let mode = (S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH) as u64;

        // Open/create the file
        let fd = syscall3(SYS_OPEN, file_path as u64, flags, mode);

        // Close the file if it was opened successfully
        if fd >= 0 {
            syscall1(SYS_CLOSE, fd as u64);
            syscall1(SYS_EXIT, 0);
        }
    }

    // Exit with error code if we couldn't create the file
    syscall1(SYS_EXIT, 1);
    loop {}
}
