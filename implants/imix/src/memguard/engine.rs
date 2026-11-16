#![allow(unsafe_op_in_unsafe_fn)]

use core::ffi::c_void;
use core::sync::atomic::{AtomicBool, Ordering};

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
const TLS_EXPANSION_SLOTS_OFFSET: usize = 0x1780;

// Process-wide singleton guard: only one MemGuard may own an encrypted state
// region at a time, so instances never cross-encrypt each other's bookkeeping.
static LIVE: AtomicBool = AtomicBool::new(false);

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
    skip_failed: bool,
    decrypt_cursor: usize,
    entries: *mut PageEntry,
    #[allow(dead_code)]
    nt_query_system_information: NtQuerySystemInformationFn,
    nt_query_information_thread: NtQueryInformationThreadFn,
    skip: [SkipRange; MAX_SKIP],
}

pub struct MemGuard {
    state: *mut State,
}

impl MemGuard {
    pub fn new() -> Option<Self> {
        if LIVE
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return None;
        }
        let ntdll = unsafe { GetModuleHandleW(NTDLL_W.as_ptr()) };
        if ntdll.is_null() {
            LIVE.store(false, Ordering::Release);
            return None;
        }
        let qsi = unsafe { GetProcAddress(ntdll, b"NtQuerySystemInformation\0".as_ptr()) };
        let qit = unsafe { GetProcAddress(ntdll, b"NtQueryInformationThread\0".as_ptr()) };
        if qsi.is_null() || qit.is_null() {
            LIVE.store(false, Ordering::Release);
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
        let skip_ok = unsafe {
            build_static_skip(
                &mut static_skip,
                &mut static_skip_count,
                nt_query_information_thread,
            )
        };
        if !skip_ok {
            LIVE.store(false, Ordering::Release);
            return None;
        }

        let page_count = unsafe { count_candidates(&static_skip, static_skip_count) };
        let capacity = page_count + CAPACITY_MARGIN;
        let header_offset = match unsafe {
            round_up(
                core::mem::size_of::<State>(),
                core::mem::align_of::<PageEntry>(),
            )
        } {
            Some(v) => v,
            None => {
                LIVE.store(false, Ordering::Release);
                return None;
            }
        };
        let region_bytes = match capacity.checked_mul(core::mem::size_of::<PageEntry>()) {
            Some(v) => v,
            None => {
                LIVE.store(false, Ordering::Release);
                return None;
            }
        };
        let region_size = match header_offset.checked_add(region_bytes) {
            Some(v) => v,
            None => {
                LIVE.store(false, Ordering::Release);
                return None;
            }
        };

        let region_base = unsafe {
            VirtualAlloc(
                core::ptr::null(),
                region_size,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_READWRITE,
            ) as *mut u8
        };
        if region_base.is_null() {
            LIVE.store(false, Ordering::Release);
            return None;
        }

        let entries = unsafe { region_base.add(header_offset) as *mut PageEntry };
        let skip_static_count = static_skip_count;

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
                    skip_failed: false,
                    decrypt_cursor: 0,
                    entries,
                    nt_query_system_information,
                    nt_query_information_thread,
                    skip: static_skip,
                },
            );
            let s = &mut *state;
            if !add_skip_region(&mut s.skip, &mut s.skip_count, region_base, region_size) {
                VirtualFree(region_base as *mut c_void, 0, MEM_RELEASE);
                LIVE.store(false, Ordering::Release);
                return None;
            }
            s.skip_static_count = s.skip_count;
        }

        Some(MemGuard {
            state: region_base as *mut State,
        })
    }

    pub fn encrypt(&mut self) -> bool {
        unsafe { encrypt_state(self.state) }
    }

    pub fn decrypt(&mut self) -> bool {
        unsafe { decrypt_state(self.state) }
    }

    pub fn add_exclusion(&mut self, base: *const u8, len: usize) -> bool {
        unsafe {
            let s = &mut *self.state;
            let ok = add_skip_region(&mut s.skip, &mut s.skip_count, base as *mut u8, len);
            if !ok {
                s.skip_failed = true;
            }
            ok
        }
    }

    pub fn clear_dynamic_exclusions(&mut self) {
        unsafe {
            let s = &mut *self.state;
            s.skip_count = s.skip_static_count;
            s.skip_failed = false;
        }
    }

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
            if s.count == 0 {
                VirtualFree(s.region_base as *mut c_void, 0, MEM_RELEASE);
                LIVE.store(false, Ordering::Release);
            }
            // Otherwise decrypt failed and pages are still PAGE_NOACCESS. The
            // region is intentionally leaked (not freed) so the VEH can still
            // consult the page list, and LIVE stays set so no instance can be
            // created that would cross-encrypt this still-encrypted region.
        }
    }
}

unsafe fn round_up(value: usize, align: usize) -> Option<usize> {
    value.checked_add(align - 1).map(|x| x & !(align - 1))
}

unsafe fn add_skip_region(
    skip: &mut [SkipRange; MAX_SKIP],
    count: &mut usize,
    base: *mut u8,
    len: usize,
) -> bool {
    if *count < MAX_SKIP && len > 0 {
        skip[*count] = SkipRange { base, len };
        *count += 1;
        true
    } else {
        false
    }
}

unsafe fn is_excluded(skip: &[SkipRange], count: usize, page: *mut u8) -> bool {
    let p = page as usize;
    let pend = p.checked_add(PAGE_SIZE).unwrap_or(usize::MAX);
    for i in 0..count {
        let s = &skip[i];
        let sb = s.base as usize;
        let se = sb.checked_add(s.len).unwrap_or(usize::MAX);
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
) -> bool {
    let pid = GetCurrentProcessId();
    let snap = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
    if snap == INVALID_HANDLE_VALUE {
        return false;
    }
    let mut te: THREADENTRY32 = core::mem::zeroed();
    te.dwSize = core::mem::size_of::<THREADENTRY32>() as u32;
    let mut ok = true;
    if Thread32First(snap, &mut te) != 0 {
        loop {
            if te.th32OwnerProcessID == pid
                && !add_thread_skip(
                    static_skip,
                    skip_count,
                    te.th32ThreadID,
                    nt_query_information_thread,
                )
            {
                ok = false;
            }
            if Thread32Next(snap, &mut te) == 0 {
                break;
            }
        }
    }
    CloseHandle(snap);
    ok
}

unsafe fn add_thread_skip(
    static_skip: &mut [SkipRange; MAX_SKIP],
    skip_count: &mut usize,
    tid: u32,
    nt_query_information_thread: NtQueryInformationThreadFn,
) -> bool {
    let h = OpenThread(THREAD_QUERY_INFORMATION, 0, tid);
    if h.is_null() {
        return false;
    }
    let mut tbi = core::mem::MaybeUninit::<ThreadBasicInformation>::uninit();
    let status = nt_query_information_thread(
        h,
        THREAD_BASIC_INFORMATION_CLASS,
        tbi.as_mut_ptr() as *mut c_void,
        core::mem::size_of::<ThreadBasicInformation>() as u32,
        core::ptr::null_mut(),
    );
    let mut ok = true;
    if status >= 0 {
        let tbi = tbi.assume_init();
        let teb = tbi.teb_base;
        if !teb.is_null() {
            let teb_size = region_size_of(teb);
            if !add_skip_region(static_skip, skip_count, teb as *mut u8, teb_size) {
                ok = false;
            }

            let peb = *(teb.add(0x60) as *const *mut c_void);
            if !peb.is_null() {
                let peb_size = region_size_of(peb);
                if !add_skip_region(static_skip, skip_count, peb as *mut u8, peb_size) {
                    ok = false;
                }
            }

            // NOTE: Phase 2 moves stack exclusion to the per-cycle path. The
            // watchdog excludes only the +-RSP_HOT_WINDOW_PAGES window around
            // each suspended thread's captured Rsp, so the rest of every stack
            // is now encrypted. Whole-stack exclusion here would prevent that,
            // so it is intentionally omitted from the static (init-time) skip.
            // The TEB, PEB, and TLS blocks below remain statically excluded.

            let tls = *(teb.add(0x58) as *const *mut c_void);
            if !tls.is_null() {
                let page = (tls as usize & !(PAGE_SIZE - 1)) as *mut u8;
                if !add_skip_region(static_skip, skip_count, page, PAGE_SIZE) {
                    ok = false;
                }
            }

            let tls_exp = *(teb.add(TLS_EXPANSION_SLOTS_OFFSET) as *const *mut c_void);
            if !tls_exp.is_null() {
                let tls_exp_size = region_size_of(tls_exp);
                if !add_skip_region(static_skip, skip_count, tls_exp as *mut u8, tls_exp_size) {
                    ok = false;
                }
            }
        }
    }
    CloseHandle(h);
    ok
}

unsafe fn encrypt_state(state: *mut State) -> bool {
    if (*state).count != 0 || (*state).skip_failed {
        return false;
    }
    // A fresh cycle always starts from a clean decrypt cursor.
    (*state).decrypt_cursor = 0;
    let capacity = (*state).capacity;
    let entries = (*state).entries;
    let mut failed = false;
    let mut stop = false;
    virtual_walk(WALK_START, |base, size, st, protect, ty| {
        if stop {
            return;
        }
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
                if is_excluded(&s.skip, s.skip_count, page) {
                    p += PAGE_SIZE;
                    continue;
                }
                let s = &mut *state;
                if s.count >= capacity {
                    failed = true;
                    stop = true;
                    break;
                }
                // Page is currently PAGE_READWRITE, so it can be mutated in place.
                let slice = core::slice::from_raw_parts_mut(page, PAGE_SIZE);
                for b in slice.iter_mut() {
                    *b ^= XOR_KEY;
                }
                let mut old = 0u32;
                if VirtualProtect(page as *const c_void, PAGE_SIZE, PAGE_NOACCESS, &mut old) != 0 {
                    let idx = s.count;
                    let e = &mut *entries.add(idx);
                    e.base = page;
                    e.orig_protect = protect;
                    s.count += 1;
                } else {
                    // Protection failed: revert to plaintext and leave the page RW
                    // so it is never left holding ciphertext.
                    let slice = core::slice::from_raw_parts_mut(page, PAGE_SIZE);
                    for b in slice.iter_mut() {
                        *b ^= XOR_KEY;
                    }
                    failed = true;
                }
                p += PAGE_SIZE;
            }
        }
    });
    !failed
}

unsafe fn decrypt_state(state: *mut State) -> bool {
    let s = &mut *state;
    let entries = s.entries;
    let n = s.count;
    // Resume from the cursor so a failed decrypt can be retried without
    // re-XORing pages that are already plaintext. Invariant for the VEH:
    // entries[..decrypt_cursor] are plaintext/accessible, entries[decrypt_cursor..]
    // are still encrypted (PAGE_NOACCESS).
    let mut i = s.decrypt_cursor;
    while i < n {
        let e = &*entries.add(i);
        let page = e.base;
        // The page is PAGE_NOACCESS; make it writable before mutating it.
        let mut old = 0u32;
        if VirtualProtect(page as *const c_void, PAGE_SIZE, PAGE_READWRITE, &mut old) == 0 {
            s.decrypt_cursor = i;
            return false;
        }
        let slice = core::slice::from_raw_parts_mut(page, PAGE_SIZE);
        for b in slice.iter_mut() {
            *b ^= XOR_KEY;
        }
        if e.orig_protect != PAGE_READWRITE {
            let mut old = 0u32;
            if VirtualProtect(page as *const c_void, PAGE_SIZE, e.orig_protect, &mut old) == 0 {
                s.decrypt_cursor = i;
                return false;
            }
        }
        i += 1;
    }
    s.decrypt_cursor = 0;
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
        // Serialize with the watchdog integration test which also creates the
        // process-wide MemGuard singleton.
        let _g = crate::memguard::TEST_LOCK.lock().unwrap();
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

            // Singleton guard: a second concurrent instance must be refused
            // while one is already alive.
            assert!(
                MemGuard::new().is_none(),
                "second MemGuard::new() must return None while one is alive"
            );

            for cycle in 0..2 {
                assert!(guard.encrypt(), "encrypt failed (cycle {cycle})");

                let mut mbi = core::mem::MaybeUninit::<MEMORY_BASIC_INFORMATION>::uninit();
                let r = VirtualQuery(
                    buf as *const c_void,
                    mbi.as_mut_ptr(),
                    core::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                );
                assert!(r != 0, "VirtualQuery failed (cycle {cycle})");
                assert_eq!(
                    mbi.assume_init().Protect,
                    PAGE_NOACCESS,
                    "buffer should be NOACCESS after encrypt (cycle {cycle})"
                );

                let mut old = 0u32;
                assert!(
                    VirtualProtect(buf as *const c_void, PAGE_SIZE, PAGE_READONLY, &mut old) != 0,
                    "VirtualProtect RO failed (cycle {cycle})"
                );
                let mut enc = [0u8; PAGE_SIZE];
                core::ptr::copy_nonoverlapping(buf, enc.as_mut_ptr(), PAGE_SIZE);
                assert!(
                    VirtualProtect(buf as *const c_void, PAGE_SIZE, PAGE_NOACCESS, &mut old) != 0,
                    "VirtualProtect NA failed (cycle {cycle})"
                );

                for i in 0..PAGE_SIZE {
                    assert_eq!(
                        enc[i],
                        pattern[i] ^ XOR_KEY,
                        "byte {i} not XOR'd (cycle {cycle})"
                    );
                }

                assert!(guard.decrypt(), "decrypt failed (cycle {cycle})");

                let mut mbi = core::mem::MaybeUninit::<MEMORY_BASIC_INFORMATION>::uninit();
                let r = VirtualQuery(
                    buf as *const c_void,
                    mbi.as_mut_ptr(),
                    core::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                );
                assert!(r != 0, "VirtualQuery failed (cycle {cycle})");
                assert_eq!(
                    mbi.assume_init().Protect,
                    PAGE_READWRITE,
                    "buffer should be READWRITE after decrypt (cycle {cycle})"
                );

                let mut dec = [0u8; PAGE_SIZE];
                core::ptr::copy_nonoverlapping(buf, dec.as_mut_ptr(), PAGE_SIZE);
                for i in 0..PAGE_SIZE {
                    assert_eq!(dec[i], pattern[i], "byte {i} not restored (cycle {cycle})");
                }
            }

            VirtualFree(buf as *mut c_void, 0, MEM_RELEASE);
        }
    }
}
