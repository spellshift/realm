#![allow(unsafe_op_in_unsafe_fn)]

use core::ffi::c_void;

use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First, Thread32Next,
};
use windows_sys::Win32::System::Memory::{
    MEM_COMMIT, MEM_PRIVATE, MEM_RELEASE, MEM_RESERVE, PAGE_GUARD, PAGE_NOACCESS, PAGE_READWRITE,
    VirtualAlloc, VirtualFree, VirtualProtect, VirtualQuery,
};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcessId, OpenThread, THREAD_QUERY_INFORMATION,
};

const PAGE_SIZE: usize = 4096;
const XOR_KEY: u8 = 0x69;
const WALK_START: usize = 0x1_0000;
const MAX_SKIP: usize = 1024;
const CAPACITY_MARGIN: usize = 256;
const THREAD_BASIC_INFORMATION_CLASS: u32 = 0;

type NtQuerySystemInformationFn = extern "system" fn(u32, *mut c_void, u32, *mut u32) -> i32;
type NtQueryInformationThreadFn =
    extern "system" fn(*mut c_void, u32, *mut c_void, u32, *mut u32) -> i32;

unsafe extern "system" {
    fn GetModuleHandleW(lpModuleName: *const u16) -> *mut c_void;
    fn GetProcAddress(hModule: *mut c_void, lpProcName: *const u8) -> *mut c_void;
}

const NTDLL_W: &[u16] = &[110, 116, 100, 108, 108, 46, 100, 108, 108, 0];

#[repr(C)]
#[derive(Clone, Copy)]
struct SkipRange {
    base: *mut u8,
    len: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct PageEntry {
    base: *mut u8,
    orig_protect: u32,
}

#[repr(C)]
struct ThreadBasicInformation {
    exit_status: i32,
    teb_base: *mut c_void,
    unique_process: *mut c_void,
    unique_thread: *mut c_void,
    affinity_mask: usize,
    priority: i32,
    base_priority: i32,
}

#[repr(C)]
struct State {
    region_base: *mut u8,
    region_size: usize,
    capacity: usize,
    count: usize,
    skip_static_count: usize,
    skip_count: usize,
    entries: *mut PageEntry,
    #[allow(dead_code)]
    nt_query_system_information: NtQuerySystemInformationFn,
    nt_query_information_thread: NtQueryInformationThreadFn,
    skip: [SkipRange; MAX_SKIP],
}

/// Core memory obfuscation guard.
///
/// All bookkeeping lives in a single unencrypted `VirtualAlloc`'d region so the
/// encrypt/decrypt paths never allocate. Construct with [`MemGuard::new`].
pub struct MemGuard {
    state: *mut State,
}

impl MemGuard {
    /// Allocate the unencrypted state region, resolve ntdll natives, enumerate
    /// the current process threads and build the static skip-list, and pre-size
    /// the encrypted-page list. Returns `None` if any step fails.
    pub fn new() -> Option<Self> {
        let ntdll = unsafe { GetModuleHandleW(NTDLL_W.as_ptr()) };
        if ntdll.is_null() {
            return None;
        }
        let qsi = unsafe { GetProcAddress(ntdll, b"NtQuerySystemInformation\0".as_ptr()) };
        let qit = unsafe { GetProcAddress(ntdll, b"NtQueryInformationThread\0".as_ptr()) };
        if qsi.is_null() || qit.is_null() {
            return None;
        }
        let nt_query_system_information: NtQuerySystemInformationFn =
            unsafe { core::mem::transmute(qsi) };
        let nt_query_information_thread: NtQueryInformationThreadFn =
            unsafe { core::mem::transmute(qit) };

        let mut static_skip: [SkipRange; MAX_SKIP] = [SkipRange {
            base: core::ptr::null_mut(),
            len: 0,
        }; MAX_SKIP];
        let mut static_skip_count: usize = 0;
        unsafe {
            build_static_skip(
                &mut static_skip,
                &mut static_skip_count,
                nt_query_information_thread,
            );
        }

        let page_count = unsafe { count_candidates(&static_skip, static_skip_count) };
        let capacity = page_count + CAPACITY_MARGIN;
        let header_offset = unsafe {
            round_up(
                core::mem::size_of::<State>(),
                core::mem::align_of::<PageEntry>(),
            )
        };
        let region_size = header_offset + capacity * core::mem::size_of::<PageEntry>();

        let region_base = unsafe {
            VirtualAlloc(
                core::ptr::null(),
                region_size,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_READWRITE,
            ) as *mut u8
        };
        if region_base.is_null() {
            return None;
        }

        let entries = unsafe { region_base.add(header_offset) as *mut PageEntry };
        let skip_static_count = static_skip_count;

        // Build the State header in place, then append the state region itself
        // to the skip-list so encrypt never touches our own bookkeeping.
        unsafe {
            let state = region_base as *mut State;
            core::ptr::write(
                state,
                State {
                    region_base,
                    region_size,
                    capacity,
                    count: 0,
                    skip_static_count,
                    skip_count: skip_static_count,
                    entries,
                    nt_query_system_information,
                    nt_query_information_thread,
                    skip: static_skip,
                },
            );
            let s = &mut *state;
            add_skip_region(&mut s.skip, &mut s.skip_count, region_base, region_size);
            s.skip_static_count = s.skip_count;
        }

        Some(MemGuard {
            state: region_base as *mut State,
        })
    }

    /// Encrypt every collected private RW page: XOR with [`XOR_KEY`] and switch
    /// the page to `PAGE_NOACCESS`. Pages in the skip-list are left untouched.
    /// Returns `false` if a previous cycle was not decrypted first.
    pub fn encrypt(&mut self) -> bool {
        unsafe { encrypt_state(self.state) }
    }

    /// Decrypt every encrypted page, restoring each to its original protection,
    /// page-by-page so a faulting thread can resume immediately.
    pub fn decrypt(&mut self) -> bool {
        unsafe { decrypt_state(self.state) }
    }

    /// Extend the skip-list with a caller-provided (base, len) exclusion. Used by
    /// later phases to protect suspended-thread stack windows and similar ranges.
    pub fn add_exclusion(&mut self, base: *const u8, len: usize) {
        unsafe {
            let s = &mut *self.state;
            add_skip_region(&mut s.skip, &mut s.skip_count, base as *mut u8, len);
        }
    }

    /// Drop any caller-provided exclusions, restoring the static skip-list.
    pub fn clear_dynamic_exclusions(&mut self) {
        unsafe {
            let s = &mut *self.state;
            s.skip_count = s.skip_static_count;
        }
    }

    /// O(log n) lookup of the encrypted-page entry covering `addr`, used by the
    /// VEH in a later phase. Returns `None` if the address is not encrypted.
    pub fn find_encrypted_page(&self, addr: *const u8) -> Option<usize> {
        unsafe {
            let s = &*self.state;
            let entries = s.entries;
            let page = (addr as usize & !(PAGE_SIZE - 1)) as *mut u8;
            let mut lo = 0usize;
            let mut hi = s.count;
            while lo < hi {
                let mid = lo + (hi - lo) / 2;
                let e = &*entries.add(mid);
                if e.base < page {
                    lo = mid + 1;
                } else if e.base > page {
                    hi = mid;
                } else {
                    return Some(mid);
                }
            }
            None
        }
    }

    /// Number of currently encrypted pages.
    pub fn encrypted_count(&self) -> usize {
        unsafe { (*self.state).count }
    }
}

impl Drop for MemGuard {
    fn drop(&mut self) {
        unsafe {
            let s = &mut *self.state;
            if s.count != 0 {
                decrypt_state(self.state);
            }
            VirtualFree(s.region_base as *mut c_void, 0, MEM_RELEASE);
        }
    }
}

unsafe fn round_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

unsafe fn add_skip_region(
    skip: &mut [SkipRange; MAX_SKIP],
    count: &mut usize,
    base: *mut u8,
    len: usize,
) {
    if *count < MAX_SKIP && len > 0 {
        skip[*count] = SkipRange { base, len };
        *count += 1;
    }
}

unsafe fn is_excluded(skip: &[SkipRange], count: usize, page: *mut u8) -> bool {
    let p = page as usize;
    let pend = p + PAGE_SIZE;
    for i in 0..count {
        let sb = skip[i].base as usize;
        let se = sb + skip[i].len;
        if p < se && sb < pend {
            return true;
        }
    }
    false
}

unsafe fn region_size_of(ptr: *mut c_void) -> usize {
    let mut mbi = core::mem::MaybeUninit::<
        windows_sys::Win32::System::Memory::MEMORY_BASIC_INFORMATION,
    >::uninit();
    let ret = VirtualQuery(
        ptr,
        mbi.as_mut_ptr(),
        core::mem::size_of::<windows_sys::Win32::System::Memory::MEMORY_BASIC_INFORMATION>(),
    );
    if ret == 0 {
        0
    } else {
        mbi.assume_init().RegionSize
    }
}

unsafe fn virtual_walk(mut addr: usize, mut visit: impl FnMut(*mut u8, usize, u32, u32, u32)) {
    let mut mbi = core::mem::MaybeUninit::<
        windows_sys::Win32::System::Memory::MEMORY_BASIC_INFORMATION,
    >::uninit();
    loop {
        let ret = VirtualQuery(
            addr as *const c_void,
            mbi.as_mut_ptr(),
            core::mem::size_of::<windows_sys::Win32::System::Memory::MEMORY_BASIC_INFORMATION>(),
        );
        if ret == 0 {
            break;
        }
        let mbi = mbi.assume_init();
        let base = mbi.BaseAddress as usize;
        let size = mbi.RegionSize;
        if base == 0 {
            break;
        }
        visit(base as *mut u8, size, mbi.State, mbi.Protect, mbi.Type);
        let next = base + size;
        if next <= addr {
            break;
        }
        addr = next;
    }
}

unsafe fn count_candidates(skip: &[SkipRange], skip_count: usize) -> usize {
    let mut count = 0usize;
    virtual_walk(WALK_START, |base, size, state, protect, ty| {
        if state == MEM_COMMIT
            && ty == MEM_PRIVATE
            && protect == PAGE_READWRITE
            && (protect & PAGE_GUARD) == 0
        {
            let mut p = base as usize;
            let end = p + size;
            while p + PAGE_SIZE <= end {
                let page = p as *mut u8;
                if !is_excluded(skip, skip_count, page) {
                    count += 1;
                }
                p += PAGE_SIZE;
            }
        }
    });
    count
}

unsafe fn build_static_skip(
    static_skip: &mut [SkipRange; MAX_SKIP],
    skip_count: &mut usize,
    nt_query_information_thread: NtQueryInformationThreadFn,
) {
    let pid = GetCurrentProcessId();
    let snap = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
    if snap == INVALID_HANDLE_VALUE {
        return;
    }
    let mut te: THREADENTRY32 = core::mem::zeroed();
    te.dwSize = core::mem::size_of::<THREADENTRY32>() as u32;
    if Thread32First(snap, &mut te) != 0 {
        loop {
            if te.th32OwnerProcessID == pid {
                add_thread_skip(
                    static_skip,
                    skip_count,
                    te.th32ThreadID,
                    nt_query_information_thread,
                );
            }
            if Thread32Next(snap, &mut te) == 0 {
                break;
            }
        }
    }
    CloseHandle(snap);
}

unsafe fn add_thread_skip(
    static_skip: &mut [SkipRange; MAX_SKIP],
    skip_count: &mut usize,
    tid: u32,
    nt_query_information_thread: NtQueryInformationThreadFn,
) {
    let h = OpenThread(THREAD_QUERY_INFORMATION, 0, tid);
    if h.is_null() {
        return;
    }
    let mut tbi = core::mem::MaybeUninit::<ThreadBasicInformation>::uninit();
    let status = nt_query_information_thread(
        h,
        THREAD_BASIC_INFORMATION_CLASS,
        tbi.as_mut_ptr() as *mut c_void,
        core::mem::size_of::<ThreadBasicInformation>() as u32,
        core::ptr::null_mut(),
    );
    if status >= 0 {
        let tbi = tbi.assume_init();
        let teb = tbi.teb_base;
        if !teb.is_null() {
            let teb_size = region_size_of(teb);
            add_skip_region(static_skip, skip_count, teb as *mut u8, teb_size);

            let peb = *(teb.add(0x60) as *const *mut c_void);
            if !peb.is_null() {
                let peb_size = region_size_of(peb);
                add_skip_region(static_skip, skip_count, peb as *mut u8, peb_size);
            }

            let stack_base = *(teb.add(0x08) as *const *mut c_void);
            let stack_limit = *(teb.add(0x10) as *const *mut c_void);
            if !stack_base.is_null()
                && !stack_limit.is_null()
                && stack_base as usize > stack_limit as usize
            {
                add_skip_region(
                    static_skip,
                    skip_count,
                    stack_limit as *mut u8,
                    stack_base as usize - stack_limit as usize,
                );
            }

            let tls = *(teb.add(0x58) as *const *mut c_void);
            if !tls.is_null() {
                let page = (tls as usize & !(PAGE_SIZE - 1)) as *mut u8;
                add_skip_region(static_skip, skip_count, page, PAGE_SIZE);
            }
        }
    }
    CloseHandle(h);
}

unsafe fn encrypt_state(state: *mut State) -> bool {
    if (*state).count != 0 {
        return false;
    }
    let capacity = (*state).capacity;
    let entries = (*state).entries;
    virtual_walk(WALK_START, |base, size, st, protect, ty| {
        if st == MEM_COMMIT
            && ty == MEM_PRIVATE
            && protect == PAGE_READWRITE
            && (protect & PAGE_GUARD) == 0
        {
            let mut p = base as usize;
            let end = p + size;
            while p + PAGE_SIZE <= end {
                let page = p as *mut u8;
                let s = &*state;
                if !is_excluded(&s.skip, s.skip_count, page) {
                    let s = &mut *state;
                    if s.count < capacity {
                        let slice = core::slice::from_raw_parts_mut(page, PAGE_SIZE);
                        for b in slice.iter_mut() {
                            *b ^= XOR_KEY;
                        }
                        let idx = s.count;
                        let e = &mut *entries.add(idx);
                        e.base = page;
                        e.orig_protect = protect;
                        let mut old = 0u32;
                        VirtualProtect(page as *const c_void, PAGE_SIZE, PAGE_NOACCESS, &mut old);
                        s.count += 1;
                    }
                }
                p += PAGE_SIZE;
            }
        }
    });
    true
}

unsafe fn decrypt_state(state: *mut State) -> bool {
    let s = &mut *state;
    let entries = s.entries;
    let n = s.count;
    for i in 0..n {
        let e = &*entries.add(i);
        let page = e.base;
        let slice = core::slice::from_raw_parts_mut(page, PAGE_SIZE);
        for b in slice.iter_mut() {
            *b ^= XOR_KEY;
        }
        let mut old = 0u32;
        VirtualProtect(page as *const c_void, PAGE_SIZE, e.orig_protect, &mut old);
    }
    s.count = 0;
    true
}

#[cfg(all(windows, feature = "mem-obfuscation", test))]
mod tests {
    use super::*;
    use windows_sys::Win32::System::Memory::{
        MEM_COMMIT, MEM_RESERVE, MEMORY_BASIC_INFORMATION, PAGE_NOACCESS, PAGE_READONLY,
        PAGE_READWRITE,
    };

    #[test]
    fn encrypt_decrypt_roundtrip() {
        unsafe {
            let buf = VirtualAlloc(
                core::ptr::null(),
                PAGE_SIZE,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_READWRITE,
            ) as *mut u8;
            assert!(!buf.is_null(), "VirtualAlloc failed");

            let mut pattern = [0u8; PAGE_SIZE];
            for (i, b) in pattern.iter_mut().enumerate() {
                *b = (i as u8).wrapping_add(1);
            }
            core::ptr::copy_nonoverlapping(pattern.as_ptr(), buf, PAGE_SIZE);

            let mut guard = MemGuard::new().expect("memguard init");

            assert!(guard.encrypt(), "encrypt failed");

            let mut mbi = core::mem::MaybeUninit::<MEMORY_BASIC_INFORMATION>::uninit();
            let r = VirtualQuery(
                buf as *const c_void,
                mbi.as_mut_ptr(),
                core::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
            );
            assert!(r != 0, "VirtualQuery failed");
            assert_eq!(
                mbi.assume_init().Protect,
                PAGE_NOACCESS,
                "buffer should be NOACCESS after encrypt"
            );

            let mut old = 0u32;
            assert!(
                VirtualProtect(buf as *const c_void, PAGE_SIZE, PAGE_READONLY, &mut old) != 0,
                "VirtualProtect RO failed"
            );
            let mut enc = [0u8; PAGE_SIZE];
            core::ptr::copy_nonoverlapping(buf, enc.as_mut_ptr(), PAGE_SIZE);
            assert!(
                VirtualProtect(buf as *const c_void, PAGE_SIZE, PAGE_NOACCESS, &mut old) != 0,
                "VirtualProtect NA failed"
            );

            for i in 0..PAGE_SIZE {
                assert_eq!(enc[i], pattern[i] ^ XOR_KEY, "byte {i} not XOR'd");
            }

            assert!(guard.decrypt(), "decrypt failed");

            let mut dec = [0u8; PAGE_SIZE];
            core::ptr::copy_nonoverlapping(buf, dec.as_mut_ptr(), PAGE_SIZE);
            for i in 0..PAGE_SIZE {
                assert_eq!(dec[i], pattern[i], "byte {i} not restored");
            }

            VirtualFree(buf as *mut c_void, 0, MEM_RELEASE);
        }
    }
}
