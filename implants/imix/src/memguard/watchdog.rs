//! Windows watchdog that drives the memguard encrypt/decrypt dormancy cycle.
//!
//! Gated to `cfg(all(windows, feature = "mem-obfuscation"))`; inert fallbacks
//! for every other target live in `mod.rs` so call sites compile unconditionally.
//!
//! Design (see `/tmp/memguard/phase-2.md` and `design-facts.md`):
//! - A single `std::thread` watchdog is spawned lazily on the first eligible
//!   sleep and lives for the process lifetime, parked on a kernel Event between
//!   cycles.
//! - The agent<->watchdog handshake uses kernel Events plus plain `.data`
//!   statics (never-encrypted, since `.data` is a `MEM_IMAGE` page). The agent
//!   stores the interval (as `f64` bits) in `INTERVAL_BITS`, pulses `CMD_EVENT`,
//!   then parks via `tokio::task::spawn_blocking(WaitForSingleObject(WAKE_EVENT))
//!   and `.await`s the JoinHandle. No `std` synchronization (mpsc/Barrier/condvar)
//!   is used across the encrypted window: those structures keep their wait
//!   bookkeeping on the user-mode heap, which is encrypted during the dormancy
//!   window, so a wait that resolves (or faults) there lands in ntdll dispatch
//!   code touching ciphertext and cannot be VEH-recovered.
//! - Each cycle the watchdog (1) re-enumerates the live thread set, (2) runs the
//!   bounded dormancy protocol (ThreadState==Waiting pre-check, suspend, Rip-in-
//!   ntdll re-verify), (3) encrypts, (4) sleeps the interval, (5) decrypts,
//!   (6) resumes threads, (7) signals the agent.
//! - During the encrypted window the watchdog must not touch the (now
//!   PAGE_NOACCESS) process heap, so all per-cycle working state lives on its
//!   stack or in the engine's pre-allocated state region, and the `print_debug`
//!   log lines are emitted through a non-allocating writer.
//! - Fail-closed: a thread/TEB/stack metadata failure, or a failure to fully
//!   decrypt, never leads to a corrupted resume. Threads are only resumed and
//!   the agent only woken after `decrypt()` has fully succeeded; otherwise the
//!   process terminates with `ExitProcess(1)` (encrypted memory is never exposed
//!   as plaintext and no suspended thread is ever resumed into corrupted state).

use core::ffi::c_void;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicPtr, AtomicU64, Ordering};
use std::thread;

use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::Debug::CONTEXT;
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
// Hot stack window kept plaintext around each suspended thread's captured Rsp.
// UNSAFE INVARIANT: when an access violation fires, the kernel dispatches it to
// KiUserExceptionDispatcher on the *faulting* thread, which builds its exception
// frame on that thread's own stack and walks the VEH chain. If the pages at/near
// Rsp (or the VEH's own working set) were encrypted/NOACCESS, the dispatcher
// itself would fault and the process would die before our handler runs. The
// 16-page window (+-64 KB) covers the dispatcher frame plus the first handler
// frames so the VEH can execute at all. Final numeric validation of the window
// size is deferred to Phase 4 on the Windows host.
const RSP_HOT_WINDOW_PAGES: usize = 16;
const DORMANCY_MAX_ATTEMPTS: usize = 5;
const DORMANCY_BACKOFF_MS: u64 = 5;
const DECRYPT_MAX_ATTEMPTS: usize = 30;
const PAGE_SIZE: usize = 4096;
const TLS_EXPANSION_SLOTS_OFFSET: usize = 0x1780;
// Bounded exclusion span for the TLS expansion block (1024 slots x 8 bytes).
const TLS_EXP_SPAN: usize = 8 * 1024;
// Per-module TLS block pointer array (TEB+0x58): walk the inline slot count.
const TLS_BLOCK_PTRS: usize = 64;
// CONTEXT_CONTROL for x64: CONTEXT_CONTROL (0x1) ORed with the CONTEXT_AMD64
// (0x0010_0000) selector bit. Using only 0x1 is the legacy x86 value and makes
// GetThreadContext return an incomplete/zeroed control context on x64, so the
// Rip re-verify reads garbage.
const CONTEXT_CONTROL: u32 = 0x0010_0001;
const THREAD_BASIC_INFORMATION_CLASS: u32 = 0;
const SYSTEM_PROCESS_INFORMATION_CLASS: u32 = 5;
const STATUS_INFO_LENGTH_MISMATCH: i32 = 0xC0000004u32 as i32;
const THREAD_STATE_WAITING: u32 = 5;
const MAX_THREADS: usize = 1024;

type NtQueryInformationThreadFn =
    extern "system" fn(*mut c_void, u32, *mut c_void, u32, *mut u32) -> i32;

type NtQuerySystemInformationFn = extern "system" fn(u32, *mut c_void, u32, *mut u32) -> i32;

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
    fn ExitProcess(uExitCode: u32) -> !;
    fn GetThreadContext(hThread: *mut c_void, lpContext: *mut CONTEXT) -> i32;
    fn NtQuerySystemInformation(
        system_information_class: u32,
        system_information: *mut c_void,
        system_information_length: u32,
        return_length: *mut u32,
    ) -> i32;
    fn CreateEventW(
        lpEventAttributes: *const c_void,
        bManualReset: i32,
        bInitialState: i32,
        lpName: *const u16,
    ) -> *mut c_void;
    fn WaitForSingleObject(hHandle: *mut c_void, dwMilliseconds: u32) -> u32;
    fn SetEvent(hEvent: *mut c_void) -> i32;
    fn CreateFileW(
        lpFileName: *const u16,
        dwDesiredAccess: u32,
        dwShareMode: u32,
        lpSecurityAttributes: *const c_void,
        dwCreationDisposition: u32,
        dwFlagsAndAttributes: u32,
        hTemplateFile: *mut c_void,
    ) -> *mut c_void;
    fn GetTickCount64() -> u64;
    fn GetLastError() -> u32;
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

// Trace-file FFI constants (print_debug staging). Only used by the trace helpers,
// which are no-ops without `print_debug`.
#[cfg(feature = "print_debug")]
const TRACE_FILE_APPEND: u32 = 0x2 | 0x4; // FILE_WRITE_DATA | FILE_APPEND_DATA (can create + append)
#[cfg(feature = "print_debug")]
const TRACE_FILE_OPEN_ALWAYS: u32 = 4; // OPEN_ALWAYS (3 would be OPEN_EXISTING)
#[cfg(feature = "print_debug")]
const TRACE_FILE_SHARE: u32 = 0x1 | 0x2; // FILE_SHARE_READ | FILE_SHARE_WRITE
#[cfg(feature = "print_debug")]
const TRACE_FILE_ATTR: u32 = 0x80; // FILE_ATTRIBUTE_NORMAL

// RAII guard: resumes any still-suspended threads and closes opened handles when
// the per-cycle slot buffer leaves scope (including early returns / skipped
// cycles).
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

struct CycleOutcome {
    pages: usize,
    skipped: bool,
}

// ---------------------------------------------------------------------------
// Kernel-event handshake state (plain module statics -> `.data`, a `MEM_IMAGE`
// page that the encrypt walk never touches). The agent and the watchdog signal
// each other exclusively through these auto-reset Events plus `INTERVAL_BITS`, so
// NO user-mode std synchronization (whose wait bookkeeping lives on the encrypted
// heap) is used across the dormancy window. See the module docs above.
//
// UNSAFE INVARIANT — auto-reset semantics: both Events are created auto-reset
// (bManualReset=FALSE) and initially unsignaled. `SetEvent` pulses exactly one
// waiter and atomically re-arms to unsignaled, so the cmd/wake handshake is a
// clean one-shot in both directions with no `ResetEvent` needed and no risk of a
// stale signal leaking into the next cycle. If the agent pulses `CMD_EVENT` before
// the watchdog has reached its wait, the Event latches the pulse (stays signaled)
// until the watchdog's `WaitForSingleObject` consumes it, so no command is lost.
const INFINITE: u32 = 0xFFFFFFFF;
static WAKE_EVENT: AtomicPtr<c_void> = AtomicPtr::new(core::ptr::null_mut());
static CMD_EVENT: AtomicPtr<c_void> = AtomicPtr::new(core::ptr::null_mut());
static INTERVAL_BITS: AtomicU64 = AtomicU64::new(0);
// Monotonic cycle sequence counters. The agent increments CMD_SEQ when it submits
// a command; the watchdog increments DONE_SEQ after each cycle completes (after
// resume + wake). The park loop only treats a WAKE as a real wake for THIS cycle
// when DONE_SEQ == the agent's submit sequence, which prevents a latched WAKE
// left over from a still-active previous cycle from releasing the next park in
// 0 ms mid-cycle.
static CMD_SEQ: AtomicU64 = AtomicU64::new(0);
static DONE_SEQ: AtomicU64 = AtomicU64::new(0);
// Liveness of the watchdog thread: `true` once it has been spawned (events
// created + thread launched), `false` if init failed. `engage_memguard_sleep`
// falls back to plain `tokio::time::sleep` when this is `false`.
static WATCHDOG_SPAWNED: OnceLock<bool> = OnceLock::new();

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

/// Eagerly-resolved QA debug knobs. Read **once** at watchdog init (strictly
/// before any `encrypt()`), so the encrypted window never calls `std::env::var`
/// (which would fault on the now-NOACCESS process environment block). Passed by
/// value into `run_cycle`; never re-parsed post-encryption.
struct DebugKnobs {
    midwake: Option<f64>,
    partial: Option<usize>,
}

#[cfg(any(test, feature = "print_debug"))]
fn resolve_debug_knobs() -> DebugKnobs {
    let midwake = std::env::var("IMIX_MEMGUARD_DEBUG_MIDWAKE_SECS")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|v| v.is_finite() && *v > 0.0);
    let partial = std::env::var("IMIX_MEMGUARD_DEBUG_PARTIAL_DECRYPT")
        .ok()
        .and_then(|s| s.parse::<usize>().ok());
    DebugKnobs { midwake, partial }
}

#[cfg(not(any(test, feature = "print_debug")))]
fn resolve_debug_knobs() -> DebugKnobs {
    DebugKnobs {
        midwake: None,
        partial: None,
    }
}

/// Engage the watchdog for one sleep cycle. Returns `true` once the cycle (which
/// may have been an unencrypted fail-safe) has completed and the agent has been
/// woken, `false` only when the watchdog could not be started (so the caller
/// should fall back to `tokio::time::sleep`).
pub async fn engage_memguard_sleep(interval_secs: f64) -> bool {
    // Handshake trace (print_debug-gated, non-allocating). Diagnoses the
    // production partial-resume self-heal wedge: it shows whether the agent
    // reaches the park, submits it, sends the cmd, and (critically) receives the
    // wake. The park path runs BEFORE the watchdog encrypts; the wake-received
    // path runs after full-or-partial decrypt. Both may touch encrypted state,
    // so keep this strictly non-allocating raw writes (trace_line uses StackBuf).
    trace_line("agent-entry", format_args!("interval={interval_secs}"));
    // Start (or reuse) the watchdog. `ensure_watchdog` is idempotent and returns
    // `Some(())` only once the kernel handshake Events exist and the thread is
    // running; any failure falls back to plain `tokio::time::sleep`.
    if ensure_watchdog().is_none() {
        trace_line("agent-fallback", format_args!("reason=no-watchdog"));
        return false;
    }
    let cmd = CMD_EVENT.load(Ordering::Acquire);
    let wake = WAKE_EVENT.load(Ordering::Acquire);
    if cmd.is_null() || wake.is_null() {
        trace_line("agent-fallback", format_args!("reason=no-events"));
        return false;
    }
    trace_line("agent-cmd-sent", format_args!("interval={interval_secs}"));
    // Park the async worker on a blocking-pool thread. The wait is a kernel Event
    // wait in 1-second slices that SELF-EXPIRES: if the wake is lost, or the
    // wait-return faults in a context the VEH cannot recover (a known partial-
    // resume edge case - the ntdll wait-return trampoline touching encrypted
    // memory), the loop gives up after `interval + 2s` and returns as if woken. A
    // lost wake therefore becomes a benign one-cycle timing hiccup instead of a
    // permanent hang. Each 1s timeout slice emits `agent-park-tick` so the VM
    // round can see whether the wake actually arrives. The closure reads the wake
    // Event from the `.data` static and captures only the interval plus the cmd
    // Event pointer (both Send), so it stays `Send`.
    trace_line(
        "agent-before-park",
        format_args!("interval={interval_secs}"),
    );
    let recv = tokio::task::spawn_blocking(move || {
        // Publish the interval and pulse the command Event from inside the
        // blocking thread, immediately before its wait. Doing this here (rather
        // than on the async worker before spawn) removes the startup race where
        // the watchdog could park on CMD_EVENT and enter dormancy before the
        // agent had written INTERVAL_BITS.
        // Claim the submit sequence BEFORE signaling: the watchdog snapshots CMD_SEQ
        // on wake, so publishing the command Event first would let it complete a
        // cycle against a stale sequence. Acquire/Release here pairs with the
        // watchdog's Acquire load of CMD_SEQ and its Release store of DONE_SEQ.
        INTERVAL_BITS.store(interval_secs.to_bits(), Ordering::Release);
        let my_seq = CMD_SEQ.fetch_add(1, Ordering::AcqRel) + 1;
        let _ = unsafe { SetEvent(CMD_EVENT.load(Ordering::Acquire)) };
        let start = unsafe { GetTickCount64() };
        let park_ms = if interval_secs.is_finite() {
            (interval_secs * 1000.0) as u64
        } else {
            0
        };
        let deadline_ms = park_ms.saturating_add(2000);
        let mut woke = false;
        loop {
            let waited = unsafe { WaitForSingleObject(WAKE_EVENT.load(Ordering::Acquire), 1000) };
            if waited == 0 {
                // A real wake for this cycle carries DONE_SEQ >= my_seq; an auto-reset
                // latch from a prior cycle can never reach my_seq, so the absolute
                // comparison is immune to CMD-pulse coalescing.
                if DONE_SEQ.load(Ordering::Acquire) >= my_seq {
                    woke = true;
                    break;
                }
            }
            // WAIT_TIMEOUT (or an unexpected code): emit a tick and keep waiting
            // until the self-expire deadline.
            let elapsed_ms = unsafe { GetTickCount64() }.wrapping_sub(start);
            trace_line("agent-park-tick", format_args!("elapsed_ms={elapsed_ms}"));
            if elapsed_ms >= deadline_ms {
                break;
            }
        }
        woke
    })
    .await;
    // wake-received: the blocking closure returned. `woke` distinguishes a real
    // watchdog pulse from a self-expire; either way the agent continues its loop.
    let woke = recv.unwrap_or(false);
    trace_line("agent-wake-received", format_args!("woke={woke}"));
    true
}

fn ensure_watchdog() -> Option<()> {
    let started = WATCHDOG_SPAWNED.get_or_init(|| {
        // Create the handshake Events once, pre-encryption (allocation/object
        // creation is fine here; the handles live in kernel space and the pointer
        // values live in `.data`). Auto-reset, initially unsignaled.
        let cmd = unsafe { CreateEventW(core::ptr::null(), 0, 0, core::ptr::null()) };
        let wake = unsafe { CreateEventW(core::ptr::null(), 0, 0, core::ptr::null()) };
        if cmd.is_null() || wake.is_null() {
            return false;
        }
        CMD_EVENT.store(cmd, Ordering::Release);
        WAKE_EVENT.store(wake, Ordering::Release);
        let ntdll = resolve_ntdll();
        let ntdll = match ntdll {
            Some(n) => n,
            None => return false,
        };
        match thread::Builder::new()
            .name("imix-memguard".into())
            .stack_size(8 * 1024 * 1024)
            .spawn(move || watchdog_main(ntdll))
        {
            Ok(_handle) => true,
            Err(_) => false,
        }
    });
    if *started { Some(()) } else { None }
}

fn watchdog_main(ntdll: (usize, usize)) {
    trace_init();
    let knobs = resolve_debug_knobs();
    let ntqit = resolve_ntquery();
    let ntqsi = resolve_ntqsi();
    let mut guard: Option<MemGuard> = MemGuard::new();
    let mut calib: Option<SpiLayout> = None;

    // Park on the command Event between cycles. The agent pulses CMD_EVENT after
    // publishing the interval in INTERVAL_BITS; we then run the cycle and pulse
    // WAKE_EVENT to release the agent's kernel wait. No mpsc anywhere.
    let cmd = CMD_EVENT.load(Ordering::Acquire);
    loop {
        if cmd.is_null() {
            return;
        }
        unsafe {
            let _ = WaitForSingleObject(cmd, INFINITE);
        }
        // Snapshot the submit sequence on every wake; the agent claims it (via
        // fetch_add) just before pulsing CMD_EVENT, so this is the sequence for the
        // cycle we are about to (re)run. Stored into DONE_SEQ before each wake
        // pulse below so the agent's `DONE_SEQ >= my_seq` check is always met.
        let seq = CMD_SEQ.load(Ordering::Acquire);
        let interval_secs = f64::from_bits(INTERVAL_BITS.load(Ordering::Acquire));
        let ntqit = match ntqit {
            Some(f) => f,
            None => {
                plain_sleep(interval_secs);
                let wake = WAKE_EVENT.load(Ordering::Acquire);
                // Publish DONE_SEQ for THIS submit before pulsing the wake, even
                // though we skipped the cycle (feature unavailable).
                DONE_SEQ.store(seq, Ordering::Release);
                if !wake.is_null() {
                    unsafe {
                        let _ = SetEvent(wake);
                    }
                }
                continue;
            }
        };
        let ntqsi = match ntqsi {
            Some(f) => f,
            None => {
                plain_sleep(interval_secs);
                let wake = WAKE_EVENT.load(Ordering::Acquire);
                DONE_SEQ.store(seq, Ordering::Release);
                if !wake.is_null() {
                    unsafe {
                        let _ = SetEvent(wake);
                    }
                }
                continue;
            }
        };
        // Per-cycle retry: if initialization failed earlier (e.g. a transient
        // thread exited between snapshot and OpenThread during the static-skip
        // build), retry MemGuard::new() here so the feature self-enables instead of
        // being permanently off. Cheap and only runs while disabled.
        if guard.is_none() {
            guard = MemGuard::new();
        }
        let g = match guard.as_mut() {
            Some(g) => g,
            None => {
                plain_sleep(interval_secs);
                let wake = WAKE_EVENT.load(Ordering::Acquire);
                // Publish DONE_SEQ for THIS submit before pulsing the wake.
                DONE_SEQ.store(seq, Ordering::Release);
                if !wake.is_null() {
                    unsafe {
                        let _ = SetEvent(wake);
                    }
                }
                continue;
            }
        };
        let _ = run_cycle(
            g,
            ntqit,
            ntqsi,
            &mut calib,
            ntdll.0,
            ntdll.1,
            interval_secs,
            &knobs,
        );
        // The cycle only returns after a full decrypt (resume+wake) or a clean
        // skip; a failed decrypt terminates the process in `ensure_decrypted`.
        let wake = WAKE_EVENT.load(Ordering::Acquire);
        // SELF-HEAL: in the PARTIAL_DECRYPT (or orphaned-claim) path run_cycle may
        // return with pages still encrypted but threads already resumed. Decouple
        // memory restoration from the agent's (possibly lost) wake: if any pages
        // remain encrypted, fully restore them now so memory is never left
        // encrypted past one cycle. Threads are already resumed by run_cycle, so
        // this is purely a safety net that complements the agent's self-expiring
        // park.
        let remaining = g.encrypted_count();
        if remaining != 0 {
            trace_line("watchdog-selfheal", format_args!("pages={remaining}"));
            #[cfg(any(test, feature = "print_debug"))]
            {
                let mut diag = crate::memguard::engine::DecryptDiag::default();
                ensure_decrypted(g, &mut diag);
                set_stdout_safe(true);
            }
            #[cfg(not(any(test, feature = "print_debug")))]
            {
                ensure_decrypted(g);
            }
        }
        // Mark this cycle complete (with the submit sequence we snapshotted on
        // wake) BEFORE pulsing the wake, so the agent's DONE_SEQ >= my_seq check
        // can never observe the wake without the matching sequence.
        DONE_SEQ.store(seq, Ordering::Release);
        if !wake.is_null() {
            unsafe {
                let _ = SetEvent(wake);
            }
        }
    }
}

fn plain_sleep(secs: f64) {
    if secs <= 0.0 || !secs.is_finite() {
        return;
    }
    // Slice-saturate: a single Sleep() is capped at u32::MAX ms (~49.7 days). For
    // very large intervals cast the total to u64 and Sleep in u32::MAX chunks so
    // the configured duration is honored instead of silently no-op'ing (which would
    // otherwise collapse to a back-to-back encrypt/decrypt loop).
    let mut remaining = (secs * 1000.0) as u64;
    while remaining > 0 {
        let chunk = remaining.min(u32::MAX as u64);
        unsafe { Sleep(chunk as u32) };
        remaining -= chunk;
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

/// Resolve ntdll's load base and image size by parsing the PE headers of the
/// already-mapped module. Used so the Rip safe-point check is "is Rip inside
/// ntdll?" rather than a brittle stub-allowlist.
fn resolve_ntdll() -> Option<(usize, usize)> {
    unsafe {
        let base = GetModuleHandleW(wstr("ntdll.dll").as_ptr()) as usize;
        if base == 0 {
            return None;
        }
        let dos = base as *const u8;
        // IMAGE_DOS_HEADER.e_lfanew is a 4-byte LE offset at 0x3C.
        let e_lfanew = core::ptr::read_unaligned(dos.add(0x3C) as *const i32) as usize;
        // Sanity-bound: the PE header lives near the head of the mapped image.
        if e_lfanew == 0 || e_lfanew >= 0x4000 {
            return None;
        }
        let pe = dos.add(e_lfanew);
        let sig = core::slice::from_raw_parts(pe, 4);
        if sig != b"PE\0\0" {
            return None;
        }
        // SizeOfImage sits at OptionalHeader+0x38; OptionalHeader starts after
        // the 4-byte signature and the 20-byte COFF FileHeader.
        let size_of_image =
            core::ptr::read_unaligned(dos.add(e_lfanew + 0x4 + 20 + 0x38) as *const u32) as usize;
        if size_of_image == 0 {
            return None;
        }
        Some((base, size_of_image))
    }
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

/// Resolve NtQuerySystemInformation from ntdll (raw FFI). The engine
/// already resolves this into its State, but re-resolving here keeps the change
/// local to the watchdog.
fn resolve_ntqsi() -> Option<NtQuerySystemInformationFn> {
    unsafe {
        let ntdll = GetModuleHandleW(wstr("ntdll.dll").as_ptr());
        if ntdll.is_null() {
            return None;
        }
        let p = GetProcAddress(ntdll, b"NtQuerySystemInformation\0".as_ptr());
        if p.is_null() {
            return None;
        }
        Some(core::mem::transmute(p))
    }
}

/// Field offsets for the canonical Win8+ x64 `SYSTEM_PROCESS_INFORMATION` /
/// `SYSTEM_THREAD_INFORMATION` layout (phnt-style). The thread tail offset is the
/// historically fragile part, so the caller validates the parsed data against a
/// ToolHelp-derived set before trusting it.
const THREAD_TID_OFFSET: usize = 0x30;
const THREAD_STATE_OFFSET: usize = 0x44;
const THREAD_ENTRY_SIZE: usize = 0x50;

/// Calibrated byte offsets into the Win8+ x64 SYSTEM_PROCESS_INFORMATION layout.
/// Self-calibrated at runtime against a ToolHelp oracle so the parse is robust
/// across Windows versions (the fixed part of this struct is version-fragile).
#[derive(Clone, Copy)]
struct SpiLayout {
    pid_offset: usize,
    threads_offset: usize,
}

/// One SystemProcessInformation snapshot, growing the buffer on
/// STATUS_INFO_LENGTH_MISMATCH. Allocation is fine here: this runs before the
/// encrypted window opens and the `Vec` is dropped before `encrypt()`.
fn query_raw(ntqsi: NtQuerySystemInformationFn) -> Option<Vec<u8>> {
    // Cap the buffer at 64 MiB so a misbehaving/garbage syscall cannot make us
    // allocate unbounded memory, and bound the number of retries.
    const MAX_BUF_LEN: usize = 64 * 1024 * 1024;
    const MAX_RETRIES: usize = 32;
    let mut buf_len = 0x40000usize;
    let mut retries = 0usize;
    let mut buf: Vec<u8> = Vec::with_capacity(buf_len);
    loop {
        buf.resize(buf_len, 0);
        let mut return_length: u32 = 0;
        let status = unsafe {
            ntqsi(
                SYSTEM_PROCESS_INFORMATION_CLASS,
                buf.as_mut_ptr() as *mut c_void,
                buf_len as u32,
                &mut return_length,
            )
        };
        if status >= 0 {
            return Some(buf);
        }
        if status == STATUS_INFO_LENGTH_MISMATCH {
            let hint = if return_length > 0 {
                return_length as usize
            } else {
                buf_len
            };
            buf_len = hint.saturating_mul(2).min(MAX_BUF_LEN);
            retries += 1;
            if retries > MAX_RETRIES || buf_len == 0 {
                return None;
            }
            continue;
        }
        return None;
    }
}

// Parse the (tid, ThreadState) pairs for `self_pid` using the calibrated
// layout. Takes the whole snapshot buffer so EVERY read offset and every
// NextEntryOffset step can be validated against `buf.len()` before dereferencing;
// a malformed buffer can therefore never be read out of bounds.
fn parse_threads_at(buf: &[u8], self_pid: u32, layout: &SpiLayout) -> Option<Vec<(u32, u32)>> {
    let base = buf.as_ptr();
    let len = buf.len();
    let mut out: Vec<(u32, u32)> = Vec::new();
    let mut offset: usize = 0;
    loop {
        // Every entry read must land within the snapshot buffer.
        if offset + 8 > len {
            return None;
        }
        let entry = unsafe { base.add(offset) };
        let next = unsafe { core::ptr::read_unaligned(entry.add(0x00) as *const u32) } as usize;
        if layout.pid_offset + 8 > len || offset + layout.pid_offset + 8 > len {
            return None;
        }
        let pid =
            unsafe { core::ptr::read_unaligned(entry.add(layout.pid_offset) as *const usize) };
        let num_threads =
            unsafe { core::ptr::read_unaligned(entry.add(0x04) as *const u32) } as usize;
        if pid == self_pid as usize {
            if num_threads == 0 {
                return Some(out);
            }
            if num_threads > 0x2000 {
                return None;
            }
            let threads_base_off = offset + layout.threads_offset;
            for i in 0..num_threads {
                let t_off = threads_base_off.checked_add(i * THREAD_ENTRY_SIZE)?;
                if t_off + THREAD_ENTRY_SIZE > len {
                    return None;
                }
                let t = unsafe { base.add(t_off) };
                let tid =
                    unsafe { core::ptr::read_unaligned(t.add(THREAD_TID_OFFSET) as *const u32) };
                let state =
                    unsafe { core::ptr::read_unaligned(t.add(THREAD_STATE_OFFSET) as *const u32) };
                out.push((tid, state));
            }
            return Some(out);
        }
        if next == 0 {
            break;
        }
        offset = offset.checked_add(next)?;
        if offset >= len {
            break;
        }
    }
    None
}

/// Snapshot the current process's thread TIDs via ToolHelp (oracle set, excluding
/// the watchdog TID). Used both for calibration and the per-cycle TID-unchanged
/// guard.
fn toolhelp_tids(self_tid: u32) -> Vec<u32> {
    let mut out: Vec<u32> = Vec::new();
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
        if snap.is_null() || snap == INVALID_HANDLE_VALUE {
            return out;
        }
        let mut te: THREADENTRY32 = core::mem::zeroed();
        te.dwSize = core::mem::size_of::<THREADENTRY32>() as u32;
        let mut first = true;
        loop {
            let ok = if first {
                first = false;
                Thread32First(snap, &mut te) != 0
            } else {
                Thread32Next(snap, &mut te) != 0
            };
            if !ok {
                break;
            }
            if te.th32OwnerProcessID == GetCurrentProcessId() && te.th32ThreadID != self_tid {
                out.push(te.th32ThreadID);
            }
        }
        CloseHandle(snap);
    }
    out
}

fn sets_equal(a: &[u32], b: &[u32]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut a = a.to_vec();
    let mut b = b.to_vec();
    a.sort_unstable();
    b.sort_unstable();
    a == b
}

/// Self-calibrate the SPI layout. Walk candidate (pid_offset, threads_offset)
/// pairs; accept the first whose parsed TID set exactly equals the ToolHelp oracle
/// and whose ThreadState values are plausible (0..=8). Fail closed on no match.
fn calibrate_spi(
    ntqsi: NtQuerySystemInformationFn,
    self_tid: u32,
    oracle: &[u32],
) -> Option<SpiLayout> {
    let self_pid = unsafe { GetCurrentProcessId() };
    let buf = query_raw(ntqsi)?;
    let base = buf.as_ptr();
    let len = buf.len();
    for pid_off in [0x40usize, 0x48, 0x50, 0x58] {
        for thr_off in (0x80usize..=0x180usize).step_by(8) {
            let mut offset: usize = 0;
            loop {
                if offset + 8 > len {
                    break;
                }
                let entry = unsafe { base.add(offset) };
                let next =
                    unsafe { core::ptr::read_unaligned(entry.add(0x00) as *const u32) } as usize;
                if pid_off + 8 > len || offset + pid_off + 8 > len {
                    break;
                }
                let pid = unsafe { core::ptr::read_unaligned(entry.add(pid_off) as *const usize) };
                let num =
                    unsafe { core::ptr::read_unaligned(entry.add(0x04) as *const u32) } as usize;
                if pid == self_pid as usize {
                    if num > 0x2000 {
                        break;
                    }
                    let tbase_off = offset + thr_off;
                    let mut tids: Vec<u32> = Vec::with_capacity(num.min(0x2000));
                    let mut plausible = true;
                    for i in 0..num {
                        let t_off = match tbase_off.checked_add(i * THREAD_ENTRY_SIZE) {
                            Some(v) => v,
                            None => {
                                plausible = false;
                                break;
                            }
                        };
                        if t_off + THREAD_ENTRY_SIZE > len {
                            plausible = false;
                            break;
                        }
                        let t = unsafe { base.add(t_off) };
                        let tid = unsafe {
                            core::ptr::read_unaligned(t.add(THREAD_TID_OFFSET) as *const u32)
                        };
                        let st = unsafe {
                            core::ptr::read_unaligned(t.add(THREAD_STATE_OFFSET) as *const u32)
                        };
                        if st > 8 {
                            plausible = false;
                            break;
                        }
                        tids.push(tid);
                    }
                    if plausible {
                        tids.retain(|&t| t != self_tid);
                        if sets_equal(&tids, oracle) {
                            return Some(SpiLayout {
                                pid_offset: pid_off,
                                threads_offset: thr_off,
                            });
                        }
                    }
                    break;
                }
                if next == 0 {
                    break;
                }
                offset = offset.checked_add(next)?;
                if offset >= len {
                    break;
                }
            }
        }
    }
    None
}

fn query_process_threads(
    ntqsi: NtQuerySystemInformationFn,
    self_pid: u32,
    layout: &SpiLayout,
) -> Option<Vec<(u32, u32)>> {
    let buf = query_raw(ntqsi)?;
    parse_threads_at(&buf, self_pid, layout)
}

/// Fail-closed runtime validation of the system snapshot: the parsed thread count
/// must match the ToolHelp-derived count, every parsed TID must be non-zero, and
/// the set must contain all our ToolHelp TIDs. We never guess.
fn validate_snapshot(snapshot: &[(u32, u32)], slots: &[ThreadSlot]) -> bool {
    if snapshot.len() != slots.len() {
        return false;
    }
    for (tid, _) in snapshot {
        if *tid == 0 {
            return false;
        }
    }
    for s in slots {
        if !snapshot.iter().any(|(t, _)| *t == s.tid) {
            return false;
        }
    }
    true
}

fn state_for(snapshot: &[(u32, u32)], tid: u32) -> Option<u32> {
    snapshot.iter().find(|(t, _)| *t == tid).map(|(_, st)| *st)
}

fn rip_in_ntdll(rip: u64, ntdll_base: usize, ntdll_size: usize) -> bool {
    let r = rip as usize;
    r >= ntdll_base && r < ntdll_base + ntdll_size
}

fn sleep_backoff(attempt: usize) {
    let ms = (DORMANCY_BACKOFF_MS * (attempt as u64 + 1)).min(64);
    unsafe { Sleep(ms as u32) };
}

/// Enumerate the current process's threads (excluding the watchdog TID) into a
/// stack buffer. Returns the live count, or `None` on any failure that prevents a
/// safe dormancy pass (we then skip the cycle rather than encrypt around a thread
/// we could not account for). Thread32First failing is a hard failure, not the
/// end of the snapshot.
///
/// Any handles from a previous enumeration still present in `buf` are closed
/// first, so re-enumeration across dormancy attempts never leaks.
fn enumerate_threads(
    self_tid: u32,
    ntqit: NtQueryInformationThreadFn,
    buf: &mut [ThreadSlot],
) -> Option<usize> {
    // Close any handles left over from a prior attempt's enumeration.
    for s in buf.iter_mut() {
        if !s.handle.is_null() {
            unsafe { CloseHandle(s.handle) };
            s.handle = core::ptr::null_mut();
        }
        *s = ThreadSlot::default();
    }

    let pid = unsafe { GetCurrentProcessId() };
    let snap = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
    if snap.is_null() || snap == INVALID_HANDLE_VALUE {
        return None;
    }

    let mut te: THREADENTRY32 = unsafe { core::mem::zeroed() };
    te.dwSize = core::mem::size_of::<THREADENTRY32>() as u32;

    let mut count = 0usize;
    let mut done_first = false;
    loop {
        let ok = if !done_first {
            unsafe { Thread32First(snap, &mut te) != 0 }
        } else {
            unsafe { Thread32Next(snap, &mut te) != 0 }
        };
        if !ok {
            if !done_first {
                // Thread32First failed: this is an enumeration failure, not a
                // clean end-of-snapshot. Fail closed.
                unsafe { CloseHandle(snap) };
                for s in buf.iter_mut() {
                    if !s.handle.is_null() {
                        unsafe { CloseHandle(s.handle) };
                        s.handle = core::ptr::null_mut();
                    }
                }
                return None;
            }
            break;
        }
        done_first = true;

        if te.th32OwnerProcessID != pid || te.th32ThreadID == self_tid {
            continue;
        }
        if count >= buf.len() {
            // Too many threads to account for: fail closed. Close the handles we
            // already opened before returning.
            unsafe { CloseHandle(snap) };
            for s in buf.iter_mut() {
                if !s.handle.is_null() {
                    unsafe { CloseHandle(s.handle) };
                    s.handle = core::ptr::null_mut();
                }
            }
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
            for s in buf.iter_mut() {
                if !s.handle.is_null() {
                    unsafe { CloseHandle(s.handle) };
                    s.handle = core::ptr::null_mut();
                }
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
    let status = unsafe {
        ntqit(
            handle,
            THREAD_BASIC_INFORMATION_CLASS,
            tbi.as_mut_ptr() as *mut c_void,
            core::mem::size_of::<ThreadBasicInformation>() as u32,
            core::ptr::null_mut(),
        )
    };
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

// Mirrors engine.rs's THREAD_BASIC_INFORMATION layout (x64), read via
// NtQueryInformationThread(ThreadBasicInformation=0).
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

/// Add TEB/PEB/TLS exclusions for every live thread. A null TEB means we cannot
/// exclude that thread's thread-local state, which is unsafe to encrypt around,
/// so we fail closed (return `false`).
fn add_thread_state_exclusions(guard: &mut MemGuard, slots: &[ThreadSlot]) -> bool {
    // Process-global PEB/PP are identical for every thread; add them once per
    // build, not once per thread (which would consume MAX_SKIP linearly).
    let mut peb_done = false;
    let mut pp_done = false;
    for s in slots {
        if s.teb.is_null() {
            return false;
        }
        let teb_size = region_size_of(s.teb);
        if teb_size > 0 && !guard.add_exclusion(s.teb as *const u8, teb_size) {
            return false;
        }
        unsafe {
            let peb = *(s.teb.add(0x60) as *const *mut c_void);
            if !peb.is_null() {
                if !peb_done {
                    let peb_size = region_size_of(peb);
                    if peb_size > 0 && !guard.add_exclusion(peb as *const u8, peb_size) {
                        return false;
                    } else {
                        peb_done = true;
                    }
                }
                // UNSAFE INVARIANT (narrow ProcessParameters exclusion): exclude only
                // the pages the RTL_USER_PROCESS_PARAMETERS struct occupies (x64 ~0x490
                // bytes from PEB+0x20), not its whole region (which holds the env
                // block).
                let pp = *(peb.add(0x20) as *const *mut c_void);
                if !pp.is_null() {
                    if !pp_done {
                        let pp_base = (pp as usize) & !(PAGE_SIZE - 1);
                        let pp_end = (pp as usize) + 0x490;
                        let pp_pages_end = (pp_end + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
                        let pp_len = pp_pages_end - pp_base;
                        if pp_len > 0 && !guard.add_exclusion(pp_base as *const u8, pp_len) {
                            return false;
                        } else {
                            pp_done = true;
                        }
                    }
                }
            }
            // TLS pointer table (TEB+0x58): table page(s) plus each pointed-to
            // per-module TLS block page. Excluding only the table leaves the blocks
            // encrypted, faulting on post-resume exception-dispatch TLS access.
            let tls = *(s.teb.add(0x58) as *const *mut c_void);
            if !tls.is_null() {
                let page = (tls as usize & !(PAGE_SIZE - 1)) as *const u8;
                if !guard.add_exclusion(page, PAGE_SIZE) {
                    return false;
                }
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
            // TLS expansion block (TEB+0x1780): bounded 8 KiB span, not the whole
            // VirtualQuery region (which would leak unrelated plaintext memory).
            let tls_exp = *(s.teb.add(TLS_EXPANSION_SLOTS_OFFSET) as *const *mut c_void);
            if !tls_exp.is_null() {
                let base = (tls_exp as usize & !(PAGE_SIZE - 1)) as *const u8;
                if !guard.add_exclusion(base, TLS_EXP_SPAN) {
                    return false;
                }
            }
        }
    }
    true
}

/// Exclude the watchdog's own thread-local state and entire stack. The watchdog
/// is Running while it owns the encrypted window, so its TEB/PEB/TLS and stack
/// must stay in the unencrypted set or it would fault on its own state.
fn add_watchdog_exclusions(guard: &mut MemGuard, ntqit: NtQueryInformationThreadFn) -> bool {
    let cur = unsafe { GetCurrentThread() };
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
            // UNSAFE INVARIANT (narrow ProcessParameters exclusion): exclude ONLY the
            // pages the RTL_USER_PROCESS_PARAMETERS struct itself occupies (x64 ~0x490
            // bytes from the PEB+0x20 pointer), NOT the whole VirtualQuery region
            // (which also holds the process environment block). The CRT/std handle
            // table lives inside the PP struct; the environment block is a separate
            // allocation referenced by PP->Environment and is DELIBERATELY left
            // encrypted and never read post-encryption (see eager-parse fix above).
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

/// Exclude only the +-RSP_HOT_WINDOW_PAGES window around a suspended thread's
/// captured Rsp. Any missing/garbled stack metadata is a fail-closed condition:
/// we never fall back to excluding the whole stack (that would leave the rest of
/// the stack encrypted while the thread resumes into it).
fn add_rsp_window_exclusion(guard: &mut MemGuard, s: &ThreadSlot) -> bool {
    if s.stack_base.is_null() || s.stack_limit.is_null() {
        return false;
    }
    let base = s.stack_limit as usize;
    let top = s.stack_base as usize;
    let rsp = s.rsp as usize;
    if rsp < base || rsp > top {
        return false;
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
        return false;
    }
    guard.add_exclusion(lo as *const u8, hi - lo)
}

enum DormancyResult {
    Ok,
    RetryExhausted,
    Fail,
}

// Bounded dormancy protocol. Re-enumerates the live thread set on every
// attempt. Suspends every non-watchdog thread that is currently Waiting and
// re-verifies each one is frozen inside ntdll (Rip captured via
// GetThreadContext). On success the TEB/PEB/TLS and RSP hot-window exclusions for
// each suspended thread are added to the engine's skip set.
//
// The engine's dynamic skip is rebuilt at the start of every attempt so per-
// attempt exclusions never accumulate. Thread/TEB/stack metadata failures are
// fail-closed: the cycle is aborted (the caller skips).
//
// Returns `(result, live_count)`.
fn run_dormancy(
    guard: &mut MemGuard,
    self_tid: u32,
    ntqit: NtQueryInformationThreadFn,
    ntqsi: NtQuerySystemInformationFn,
    layout: &SpiLayout,
    ntdll_base: usize,
    ntdll_size: usize,
    buf: &mut [ThreadSlot],
) -> (DormancyResult, usize) {
    for attempt in 0..DORMANCY_MAX_ATTEMPTS {
        // Rebuild the skip set from scratch each attempt.
        guard.clear_dynamic_exclusions();
        if !add_watchdog_exclusions(guard, ntqit) {
            return (DormancyResult::Fail, 0);
        }

        let n = match enumerate_threads(self_tid, ntqit, buf) {
            Some(n) => n,
            None => return (DormancyResult::Fail, 0),
        };

        if !add_thread_state_exclusions(guard, &buf[..n]) {
            resume_all(&mut buf[..n]);
            return (DormancyResult::Fail, 0);
        }

        // Authoritative thread-state comes from NtQuerySystemInformation
        // (SystemProcessInformation). Per-thread NtQueryInformationThread(0x20) is
        // invalid on the target (0x20 == ThreadCounterProfiling), so the system
        // snapshot is the only reliable source. Drop our own TID (present in the
        // snapshot but not in the ToolHelp candidate set).
        let self_pid = unsafe { GetCurrentProcessId() };
        let snapshot = match query_process_threads(ntqsi, self_pid, layout) {
            Some(s) => s
                .into_iter()
                .filter(|(tid, _)| *tid != self_tid)
                .collect::<Vec<_>>(),
            None => {
                resume_all(&mut buf[..n]);
                sleep_backoff(attempt);
                continue;
            }
        };
        if !validate_snapshot(&snapshot, &buf[..n]) {
            resume_all(&mut buf[..n]);
            sleep_backoff(attempt);
            continue;
        }

        // Pre-check: every candidate thread must report ThreadState == Waiting.
        let mut all_waiting = true;
        for s in buf[..n].iter() {
            if state_for(&snapshot, s.tid) != Some(THREAD_STATE_WAITING) {
                all_waiting = false;
                break;
            }
        }
        trace_line(
            "dormancy-attempt",
            format_args!("i={attempt} all_waiting={all_waiting}"),
        );
        if !all_waiting {
            sleep_backoff(attempt);
            continue;
        }

        // Suspend every candidate. Previous suspend count must be 0 (we must not
        // disturb a thread already suspended by a debugger).
        let mut suspend_ok = true;
        for s in buf[..n].iter_mut() {
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
        trace_line("suspended", format_args!("n={n}"));
        if !suspend_ok {
            resume_all(&mut buf[..n]);
            sleep_backoff(attempt);
            continue;
        }

        // Re-verify: each suspended thread must be frozen inside ntdll.
        let mut verify_ok = true;
        for s in buf[..n].iter_mut() {
            let mut ctx: CONTEXT = unsafe { core::mem::zeroed() };
            ctx.ContextFlags = CONTEXT_CONTROL;
            let ok = unsafe { GetThreadContext(s.handle, &mut ctx) } != 0;
            if !ok || !rip_in_ntdll(ctx.Rip, ntdll_base, ntdll_size) {
                verify_ok = false;
                break;
            }
            s.rsp = ctx.Rsp;
        }
        if !verify_ok {
            resume_all(&mut buf[..n]);
            sleep_backoff(attempt);
            continue;
        }
        trace_line("verified-ok", format_args!("n={n}"));

        // All dormant: add the RSP hot window for each suspended thread. A failure
        // here is fail-closed (we must not encrypt around a thread whose in-use
        // stack window we could not exclude).
        for s in buf[..n].iter() {
            if !add_rsp_window_exclusion(guard, s) {
                resume_all(&mut buf[..n]);
                return (DormancyResult::Fail, 0);
            }
        }
        trace_line("rsp-exclusions", format_args!("n={n}"));
        return (DormancyResult::Ok, n);
    }
    (DormancyResult::RetryExhausted, 0)
}

fn resume_all(slots: &mut [ThreadSlot]) {
    for s in slots.iter_mut() {
        if s.suspended {
            unsafe { ResumeThread(s.handle) };
            s.suspended = false;
        }
    }
}

/// Snapshot the current live thread TIDs (excluding the watchdog) and verify the
/// set matches `slots`. If a thread was created or destroyed between the
/// successful dormancy pass and here, the in-flight thread set is not what we
/// just excluded around, so the caller must abort the cycle.
fn thread_set_unchanged(self_tid: u32, slots: &[ThreadSlot]) -> bool {
    // Fixed stack buffers: this runs while every other thread is suspended, so a
    // Vec allocation here could deadlock on the allocator lock held by a
    // suspended thread. No heap is touched on this path.
    let mut cur = [0u32; MAX_THREADS];
    let mut cur_count = 0usize;
    let mut total = 0usize;
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
        if snap.is_null() || snap == INVALID_HANDLE_VALUE {
            return false;
        }
        let mut te: THREADENTRY32 = core::mem::zeroed();
        te.dwSize = core::mem::size_of::<THREADENTRY32>() as u32;
        let mut first = true;
        loop {
            let ok = if first {
                first = false;
                Thread32First(snap, &mut te) != 0
            } else {
                Thread32Next(snap, &mut te) != 0
            };
            if !ok {
                break;
            }
            if te.th32OwnerProcessID == GetCurrentProcessId() && te.th32ThreadID != self_tid {
                total += 1;
                if cur_count < MAX_THREADS {
                    cur[cur_count] = te.th32ThreadID;
                    cur_count += 1;
                }
            }
        }
        CloseHandle(snap);
    }
    if total != slots.len() {
        return false;
    }
    if cur_count != slots.len() {
        return false;
    }
    let mut a = [0u32; MAX_THREADS];
    let mut a_count = 0usize;
    for s in slots.iter() {
        a[a_count] = s.tid;
        a_count += 1;
    }
    cur[..cur_count].sort_unstable();
    a[..a_count].sort_unstable();
    cur[..cur_count] == a[..a_count]
}

/// Decrypt until fully complete. Retries with backoff; if it still fails the
/// process terminates with `ExitProcess(1)` so encrypted memory is never exposed
/// as plaintext and no suspended thread is resumed into corrupted state.
#[cfg(any(test, feature = "print_debug"))]
fn ensure_decrypted(guard: &mut MemGuard, diag: &mut crate::memguard::engine::DecryptDiag) {
    for attempt in 0..DECRYPT_MAX_ATTEMPTS {
        trace_line("decrypt-attempt", format_args!("attempt={attempt}"));
        if guard.decrypt_with_diag(diag) {
            return;
        }
        sleep_backoff(attempt);
    }
    // Final attempt; if pages remain encrypted, die rather than resume corrupted.
    if guard.encrypted_count() != 0 {
        unsafe { ExitProcess(1) };
    }
}
#[cfg(not(any(test, feature = "print_debug")))]
fn ensure_decrypted(guard: &mut MemGuard) {
    for attempt in 0..DECRYPT_MAX_ATTEMPTS {
        trace_line("decrypt-attempt", format_args!("attempt={attempt}"));
        if guard.decrypt() {
            return;
        }
        sleep_backoff(attempt);
    }
    // Final attempt; if pages remain encrypted, die rather than resume corrupted.
    if guard.encrypted_count() != 0 {
        unsafe { ExitProcess(1) };
    }
}
fn run_cycle(
    guard: &mut MemGuard,
    ntqit: NtQueryInformationThreadFn,
    ntqsi: NtQuerySystemInformationFn,
    calib: &mut Option<SpiLayout>,
    ntdll_base: usize,
    ntdll_size: usize,
    interval_secs: f64,
    knobs: &DebugKnobs,
) -> CycleOutcome {
    let self_tid = unsafe { GetCurrentThreadId() };

    trace_line("cmd-received", format_args!("interval={interval_secs}"));

    if calib.is_none() {
        let oracle = toolhelp_tids(self_tid);
        match calibrate_spi(ntqsi, self_tid, &oracle) {
            Some(layout) => {
                emit_calib(layout.pid_offset, layout.threads_offset);
                trace_line(
                    "calib-done",
                    format_args!(
                        "pid_off=0x{:x} threads_off=0x{:x}",
                        layout.pid_offset, layout.threads_offset
                    ),
                );
                *calib = Some(layout);
            }
            None => {
                emit_skip("layout calibration failed");
                trace_line(
                    "cycle-return",
                    format_args!("skipped=true reason=calib-failed"),
                );
                plain_sleep(interval_secs);
                return CycleOutcome {
                    pages: 0,
                    skipped: true,
                };
            }
        }
    }
    let layout = calib.unwrap();

    let mut slot_buf = [ThreadSlot::default(); MAX_THREADS];
    let mut slots = SlotsGuard {
        slots: &mut slot_buf,
    };

    let (dormancy, n) = run_dormancy(
        guard,
        self_tid,
        ntqit,
        ntqsi,
        &layout,
        ntdll_base,
        ntdll_size,
        slots.slots,
    );
    match dormancy {
        DormancyResult::Ok => {}
        DormancyResult::RetryExhausted => {
            emit_skip("running threads");
            trace_line(
                "cycle-return",
                format_args!("skipped=true reason=running-threads"),
            );
            plain_sleep(interval_secs);
            return CycleOutcome {
                pages: 0,
                skipped: true,
            };
        }
        DormancyResult::Fail => {
            emit_skip("dormancy failed");
            trace_line(
                "cycle-return",
                format_args!("skipped=true reason=dormancy-failed"),
            );
            plain_sleep(interval_secs);
            return CycleOutcome {
                pages: 0,
                skipped: true,
            };
        }
    }

    // Guard against threads created/removed between the successful pass and here.
    if !thread_set_unchanged(self_tid, &slots.slots[..n]) {
        resume_all(&mut slots.slots[..n]);
        emit_skip("thread set changed");
        trace_line(
            "cycle-return",
            format_args!("skipped=true reason=thread-set-changed"),
        );
        plain_sleep(interval_secs);
        return CycleOutcome {
            pages: 0,
            skipped: true,
        };
    }
    trace_line("threadset-check", format_args!("ok"));

    let t0_enc = trace_now();
    trace_line(
        "encrypt-start",
        format_args!("skip_count={}", guard.skip_count()),
    );
    let mut last = 0usize;
    let encrypt_ok = guard.encrypt_with_progress(&mut |done: usize| {
        if done / 10000 != last / 10000 {
            last = done;
            trace_line("encrypt-progress", format_args!("pages_done={done}"));
        }
    });
    let enc_elapsed = trace_now() - t0_enc;
    if !encrypt_ok {
        #[cfg(any(test, feature = "print_debug"))]
        {
            let mut diag = crate::memguard::engine::DecryptDiag::default();
            ensure_decrypted(guard, &mut diag);
            set_stdout_safe(true);
        }
        #[cfg(not(any(test, feature = "print_debug")))]
        {
            ensure_decrypted(guard);
            set_stdout_safe(true);
        }
        // Resume all suspended threads (the SlotsGuard would otherwise keep them
        // suspended until the function returns, i.e. across the plain_sleep
        // below) before emitting the skip and sleeping, so they are never left
        // suspended during the sleep.
        drop(slots);
        emit_skip("encrypt failed");
        trace_line(
            "cycle-return",
            format_args!("skipped=true reason=encrypt-failed"),
        );
        plain_sleep(interval_secs);
        return CycleOutcome {
            pages: 0,
            skipped: true,
        };
    }
    // INVARIANT (post-encrypt): from here until the cycle ends this thread touches
    // only its own stack, the already-allocated unencrypted state region, the
    // trace-file handle, and raw FFI (Sleep/GetTickCount64/WriteFile/VirtualProtect/
    // Suspend/Resume/GetThreadContext). It must NOT call std::env::var,
    // GetStdHandle, CRT locale/timezone, or allocate from the now-NOACCESS heap.
    // Debug-knob env reads were resolved by resolve_debug_knobs() at watchdog init,
    // strictly before encrypt(), so none happen here.
    set_stdout_safe(false);
    trace_line(
        "encrypt-done",
        format_args!(
            "pages={} elapsed_ms={}",
            guard.encrypted_count(),
            enc_elapsed
        ),
    );
    trace_line(
        "exclusions-done",
        format_args!("skip_count={}", guard.skip_count()),
    );

    let pages = guard.encrypted_count();
    let dormant = slots.slots[..n].iter().filter(|s| s.suspended).count();
    emit_engage(dormant, pages, interval_secs);
    trace_line("engage-emitted", format_args!("pages={pages}"));

    // --- QA debug knobs (eagerly resolved pre-encryption; see resolve_debug_knobs) -
    // MIDWAKE: shorten the encrypted sleep window to S seconds (S < interval).
    let mid = knobs.midwake.filter(|&s| s > 0.0 && s < interval_secs);

    let encrypted_sleep = match mid {
        Some(s) => {
            // Still encrypted here: emit via the non-allocating writer.
            emit_midwake(s);
            s
        }
        None => interval_secs,
    };
    trace_line("sleep-start", format_args!("secs={encrypted_sleep}"));
    plain_sleep(encrypted_sleep);
    trace_line("sleep-done", format_args!("secs={encrypted_sleep}"));

    // PARTIAL_DECRYPT: at wake, skip the full decrypt and leave (N - K) pages
    // encrypted so the resumed runtime exercises the VEH lazy-decrypt path live.
    // K=0 leaves ALL pages encrypted. QA fault-injection only; the self-heal path
    // is the NEXT cycle's encrypt() refusing (count != 0 -> "encrypt failed"
    // skip) followed by ensure_decrypted completing the recovery.
    let partial = knobs.partial;

    let t0_dec = trace_now();
    #[cfg(any(test, feature = "print_debug"))]
    {
        let (enc0, busy0, plain0) = guard.decrypt_histogram();
        trace_line(
            "decrypt-start",
            format_args!("enc={enc0} busy={busy0} plain={plain0} partial={partial:?}"),
        );
    }
    #[cfg(not(any(test, feature = "print_debug")))]
    trace_line("decrypt-start", format_args!("partial={:?}", partial));
    match partial {
        Some(k) => {
            #[cfg(any(test, feature = "print_debug"))]
            crate::memguard::debug_simulate_crash_partial_decrypt(k);
            // Pages still encrypted here: non-allocating writer.
            emit_partial(k);
        }
        None => {
            // Decrypt must fully succeed before any thread is resumed or the agent
            // woken. A failed decrypt terminates the process.
            #[cfg(any(test, feature = "print_debug"))]
            {
                let mut diag = crate::memguard::engine::DecryptDiag::default();
                ensure_decrypted(guard, &mut diag);
                set_stdout_safe(true);
                trace_line(
                    "decrypt-done",
                    format_args!(
                        "elapsed_ms={} enc={} busy={} plain={} xored={}",
                        trace_now() - t0_dec,
                        diag.enc,
                        diag.busy,
                        diag.plain,
                        diag.xored
                    ),
                );
            }
            #[cfg(not(any(test, feature = "print_debug")))]
            {
                ensure_decrypted(guard);
                set_stdout_safe(true);
                trace_line(
                    "decrypt-done",
                    format_args!("elapsed_ms={}", trace_now() - t0_dec),
                );
            }
        }
    }

    let resumed = slots.slots[..n].iter().filter(|s| s.suspended).count();
    emit_wake(pages, resumed);
    trace_line("wake-sent", format_args!("pages={pages}"));
    // Resume threads now (SlotsGuard Drop) so the remaining unencrypted sleep
    // below, if any, runs with the runtime live.
    drop(slots);
    trace_line("resumed", format_args!("n={n}"));
    // MIDWAKE remaining unencrypted sleep preserves beacon timing; the agent is
    // woken after the full interval by the caller.
    if let Some(s) = mid {
        let remaining = interval_secs - s;
        if remaining > 0.0 {
            plain_sleep(remaining);
        }
    }
    trace_line("cycle-return", format_args!("skipped=false pages={pages}"));
    CycleOutcome {
        pages,
        skipped: false,
    }
}

// ---------------------------------------------------------------------------
// Stage tracing to a dedicated file (independent of stdout plumbing). Opened once
// at watchdog init; never closed for the process lifetime. Writes are
// non-allocating (stack buffer) so they are safe both before and after the heap is
// encrypted. When `print_debug` is off, `trace_init`/`trace_line`/`trace_now` are
// no-ops and nothing is opened. This exists because a blocked stdout pipe under a
// session-0 scheduled-task launch can stall `dbg_print`'s WriteFile; the trace
// file (a real file, not a pipe) cannot block on a reader.
#[cfg(feature = "print_debug")]
static TRACE_HANDLE: core::sync::atomic::AtomicPtr<c_void> =
    core::sync::atomic::AtomicPtr::new(core::ptr::null_mut());
#[cfg(feature = "print_debug")]
static TRACE_T0: OnceLock<u64> = OnceLock::new();

// Tracks whether it is safe to write to stdout (i.e. nothing is encrypted). Set
// false the moment encryption succeeds, true again only after a full decrypt.
// Used so the stdout fallback (only when the trace file could not be opened)
// never blocks on a wedged inherited handle while memory is encrypted.
#[cfg(feature = "print_debug")]
static STDOUT_SAFE: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(true);

#[cfg(feature = "print_debug")]
fn set_stdout_safe(v: bool) {
    STDOUT_SAFE.store(v, core::sync::atomic::Ordering::Release);
}
#[cfg(not(feature = "print_debug"))]
fn set_stdout_safe(_v: bool) {}

// Non-allocating `print_debug` output. During the encrypted window the process
// heap is PAGE_NOACCESS, so a normal `println!`/`log::info!` (which allocate)
// would fault. These lines are written straight to stdout via WriteFile using a
// stack buffer.
// ---------------------------------------------------------------------------

#[cfg(feature = "print_debug")]
struct StackBuf {
    buf: [u8; 256],
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
    // Preferred path: the dedicated trace file never blocks on a reader, so it is
    // always safe to write even while the heap is encrypted.
    let th = TRACE_HANDLE.load(core::sync::atomic::Ordering::Acquire);
    if !th.is_null() {
        let mut written = 0u32;
        unsafe {
            let _ = WriteFile(
                th,
                b.buf.as_ptr() as *const c_void,
                b.len as u32,
                &mut written,
                core::ptr::null_mut(),
            );
        }
        return;
    }
    // Fallback only when the trace file could not be opened: never block on the
    // inherited stdout handle while memory is encrypted (a wedged pipe would hang
    // the cycle). Drop the line silently instead.
    if !STDOUT_SAFE.load(core::sync::atomic::Ordering::Acquire) {
        return;
    }
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
        buf: [0u8; 256],
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
        buf: [0u8; 256],
        len: 0,
    };
    let _ = core::fmt::write(&mut b, format_args!("memguard skip: {reason}\n"));
    dbg_print(&b);
}

#[cfg(feature = "print_debug")]
fn emit_wake(pages: usize, resumed: usize) {
    let mut b = StackBuf {
        buf: [0u8; 256],
        len: 0,
    };
    let _ = core::fmt::write(
        &mut b,
        format_args!("memguard wake: pages_decrypted={pages} resumed={resumed}\n"),
    );
    dbg_print(&b);
}

#[cfg(feature = "print_debug")]
fn emit_calib(pid_off: usize, threads_off: usize) {
    let mut b = StackBuf {
        buf: [0u8; 256],
        len: 0,
    };
    let _ = core::fmt::write(
        &mut b,
        format_args!("memguard calib: pid_off=0x{pid_off:x} threads_off=0x{threads_off:x}\n"),
    );
    dbg_print(&b);
}

#[cfg(not(feature = "print_debug"))]
fn emit_calib(_pid_off: usize, _threads_off: usize) {}

#[cfg(not(feature = "print_debug"))]
fn emit_engage(_dormant: usize, _pages: usize, _interval: f64) {}

#[cfg(not(feature = "print_debug"))]
fn emit_skip(_reason: &str) {}

#[cfg(not(feature = "print_debug"))]
fn emit_wake(_pages: usize, _resumed: usize) {}

#[cfg(feature = "print_debug")]
fn emit_midwake(secs: f64) {
    let mut b = StackBuf {
        buf: [0u8; 256],
        len: 0,
    };
    let _ = core::fmt::write(&mut b, format_args!("memguard debug: midwake after "));
    fmt_secs(&mut b, secs);
    let _ = core::fmt::write(&mut b, format_args!("s\n"));
    dbg_print(&b);
}
#[cfg(not(feature = "print_debug"))]
fn emit_midwake(_secs: f64) {}

#[cfg(feature = "print_debug")]
fn emit_partial(k: usize) {
    let mut b = StackBuf {
        buf: [0u8; 256],
        len: 0,
    };
    let _ = core::fmt::write(
        &mut b,
        format_args!("memguard debug: partial decrypt k={k}\n"),
    );
    dbg_print(&b);
}
#[cfg(not(feature = "print_debug"))]
fn emit_partial(_k: usize) {}

#[cfg(feature = "print_debug")]
fn trace_init() {
    TRACE_T0.get_or_init(|| unsafe { GetTickCount64() });
    if !TRACE_HANDLE
        .load(core::sync::atomic::Ordering::Acquire)
        .is_null()
    {
        return;
    }
    let name: Vec<u16> = "memguard-trace.log"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let h = unsafe {
        CreateFileW(
            name.as_ptr(),
            TRACE_FILE_APPEND,
            TRACE_FILE_SHARE,
            core::ptr::null(),
            TRACE_FILE_OPEN_ALWAYS,
            TRACE_FILE_ATTR,
            core::ptr::null_mut(),
        )
    };
    if (h as usize) == usize::MAX || h.is_null() {
        let gle = unsafe { GetLastError() };
        emit_trace_init_failed(gle);
        TRACE_HANDLE.store(core::ptr::null_mut(), core::sync::atomic::Ordering::Release);
    } else {
        TRACE_HANDLE.store(h, core::sync::atomic::Ordering::Release);
    }
}

#[cfg(feature = "print_debug")]
fn emit_trace_init_failed(gle: u32) {
    // Pre-encryption, so writing to stdout is safe (STDOUT_SAFE is still true).
    use windows_sys::Win32::System::Console::{GetStdHandle, STD_OUTPUT_HANDLE};
    let mut b = StackBuf {
        buf: [0u8; 256],
        len: 0,
    };
    let _ = core::fmt::write(
        &mut b,
        format_args!("memguard trace-init failed gle={gle}\n"),
    );
    let h = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
    if (h as usize) != usize::MAX {
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
}
#[cfg(not(feature = "print_debug"))]
fn trace_init() {}

#[cfg(feature = "print_debug")]
fn trace_now() -> u64 {
    let t0 = *TRACE_T0.get().unwrap_or(&0);
    unsafe { GetTickCount64() }.wrapping_sub(t0)
}
#[cfg(not(feature = "print_debug"))]
fn trace_now() -> u64 {
    0
}

#[cfg(feature = "print_debug")]
fn trace_line(stage: &str, extra: core::fmt::Arguments<'_>) {
    let elapsed = trace_now();
    let mut b = StackBuf {
        buf: [0u8; 256],
        len: 0,
    };
    let _ = core::fmt::write(&mut b, format_args!("[{elapsed:>8}ms] {stage} "));
    let _ = core::fmt::write(&mut b, extra);
    let _ = core::fmt::write(
        &mut b,
        format_args!(
            "
"
        ),
    );
    let h = TRACE_HANDLE.load(core::sync::atomic::Ordering::Acquire);
    if !h.is_null() {
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
}
#[cfg(not(feature = "print_debug"))]
fn trace_line(_stage: &str, _extra: core::fmt::Arguments<'_>) {}

// ---------------------------------------------------------------------------
// VEH recovery / miss trace (print_debug only). Called from the engine's VEH
// handler (engine.rs) so the diagnosis trace can localize the production partial-
// resume self-heal wedge. Both lines are written through the same non-allocating
// `dbg_print` machinery (trace file when open, else the safe stdout fallback),
// matching the `veh-recover` / `veh-miss` formats consumed by the VM round.
// ---------------------------------------------------------------------------
#[cfg(feature = "print_debug")]
pub(crate) fn trace_veh_recover(addr: usize, n: usize) {
    let mut b = StackBuf {
        buf: [0u8; 256],
        len: 0,
    };
    let _ = core::fmt::write(&mut b, format_args!("veh-recover page=0x{addr:x} n={n}\n"));
    dbg_print(&b);
}

#[cfg(feature = "print_debug")]
pub(crate) fn trace_veh_miss(addr: usize, code: usize) {
    let mut b = StackBuf {
        buf: [0u8; 256],
        len: 0,
    };
    let _ = core::fmt::write(
        &mut b,
        format_args!("veh-miss addr=0x{addr:x} code=0x{code:x}\n"),
    );
    dbg_print(&b);
}

// ---------------------------------------------------------------------------
// Windows-gated integration test: one full dormancy + encrypt + decrypt +
// resume cycle against real memory, with a parked thread that must wake.
// Runs on Windows CI / the Phase 4 host.
// ---------------------------------------------------------------------------

#[cfg(all(windows, feature = "mem-obfuscation", test))]
mod tests {
    use super::*;
    use core::ffi::c_void;
    use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
    use std::time::Duration;
    use windows_sys::Win32::System::Memory::{
        MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE, VirtualAlloc, VirtualFree,
    };

    // Spin-wait handshake for windows_veh_fault_recovery. Module-level `.data`
    // statics (never encrypted by the memguard walk) so the spawned touching
    // threads and the test thread coordinate WITHOUT any kernel wait — a kernel
    // wait that completes while memory is encrypted faults in the ntdll wait-
    // return trampoline (NtWaitForSingleObject post-syscall), a context the VEH
    // structurally cannot intercept. The go flag is a plain atomic the threads
    // BUSY-spin on; results are written to atomics; the test thread BUSY-spin-
    // waits on them. Serialized with the other memguard tests via TEST_LOCK, so
    // reuse across runs is safe.
    static VEH_TEST_GO: AtomicBool = AtomicBool::new(false);
    // Result codes: 0 = pending, 1 = ok (VEH recovered and value matched),
    // 2 = faulted but value mismatched.
    static VEH_TEST_RES_A: AtomicU8 = AtomicU8::new(0);
    static VEH_TEST_RES_B: AtomicU8 = AtomicU8::new(0);

    // Minimal FFI for the recovery test: a non-blocking clock read used only for
    // the spin-wait deadline. No std synchronization (kernel Event/Barrier/
    // mpsc/condvar) is used across the encrypted window.
    unsafe extern "system" {
        fn GetTickCount64() -> u64;
    }

    // Allocate a private RW page (page-aligned) for fault-recovery tests.
    unsafe fn alloc_page() -> *mut u8 {
        VirtualAlloc(
            core::ptr::null(),
            PAGE_SIZE,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        ) as *mut u8
    }

    #[test]
    fn windows_dormancy_cycle() {
        // Serialize with the engine's round-trip test which also creates the
        // MemGuard singleton.
        let _g = crate::memguard::TEST_LOCK.lock().unwrap();

        // Park a thread inside ntdll (NtDelayExecution) so the dormancy
        // protocol's Rip-in-ntdll re-verify accepts it as dormant.
        let parked = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(1000));
        });

        let ntqit = resolve_ntquery().expect("NtQueryInformationThread unavailable");
        let ntqsi = resolve_ntqsi().expect("NtQuerySystemInformation unavailable");
        let (ntdll_base, ntdll_size) = resolve_ntdll().expect("ntdll resolve");
        let mut guard = MemGuard::new().expect("memguard init");

        let mut calib: Option<SpiLayout> = None;
        let outcome = run_cycle(
            &mut guard,
            ntqit,
            ntqsi,
            &mut calib,
            ntdll_base,
            ntdll_size,
            0.0,
            &DebugKnobs {
                midwake: None,
                partial: None,
            },
        );
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

        parked.join().expect("parked thread joined");
    }

    // VEH decrypt-on-fault recovery. Simulate a watchdog crash mid-bulk-decrypt
    // (leaving pages NOACCESS+encrypted), then spawn TWO dedicated threads that
    // each touch a distinct encrypted page. Each access violation must be
    // handled by the VEH, which lazily decrypts the page so the thread survives
    // and reads plaintext. The two threads fault concurrently, proving (review
    // fix #4) that different-thread faults are both admitted and recover — not
    // serialized behind a global flag. The remaining encrypted pages must then
    // still be recoverable via a normal bulk decrypt, and subsequent
    // encrypt/decrypt cycles must keep working.
    #[test]
    fn windows_veh_fault_recovery() {
        // Serialize with the other memguard tests which also own the singleton.
        let _g = crate::memguard::TEST_LOCK.lock().unwrap();

        // Reset the spin-wait handshake statics so repeated runs (the test suite
        // runs this test more than once) start from fresh state.
        VEH_TEST_GO.store(false, Ordering::Release);
        VEH_TEST_RES_A.store(0, Ordering::Release);
        VEH_TEST_RES_B.store(0, Ordering::Release);

        let mut guard = MemGuard::new().expect("memguard init");

        // Two private RW pages to fault on (two distinct encrypted pages).
        let buf_a = unsafe { alloc_page() };
        let buf_b = unsafe { alloc_page() };
        assert!(!buf_a.is_null() && !buf_b.is_null(), "VirtualAlloc failed");

        let mut pattern_a = [0u8; PAGE_SIZE];
        let mut pattern_b = [0u8; PAGE_SIZE];
        for (i, (a, b)) in pattern_a.iter_mut().zip(pattern_b.iter_mut()).enumerate() {
            *a = (i as u8).wrapping_add(1);
            *b = (i as u8).wrapping_add(2);
        }
        unsafe {
            core::ptr::copy_nonoverlapping(pattern_a.as_ptr(), buf_a, PAGE_SIZE);
            core::ptr::copy_nonoverlapping(pattern_b.as_ptr(), buf_b, PAGE_SIZE);
        }

        // Pre-encrypt TEB-bounds reporting stays mpsc (plaintext). The spawned
        // threads report their own TEB + stack bounds so the test can exclude
        // them; everything that runs DURING the encrypted window is a BUSY
        // spin-wait on `.data` atomics (no kernel wait), so faults come from
        // RUNNING code (read_volatile) — the context the VEH is designed for.
        let (bounds_tx, bounds_rx) = std::sync::mpsc::channel::<(usize, usize, usize)>();
        // (teb, stack_base, stack_limit)

        let expected_a = pattern_a[0];
        let expected_b = pattern_b[0];
        let pa = buf_a as usize;
        let pb = buf_b as usize;

        let tx_a = bounds_tx.clone();
        let _ = std::thread::spawn(move || {
            // Report this thread's own TEB + stack bounds before any encryption.
            let teb: usize = unsafe {
                let t: usize;
                core::arch::asm!("mov {0}, gs:[0x30]", out(reg) t, options(nostack));
                t
            };
            let stack_base = unsafe { *((teb as *const u8).add(0x08) as *const *mut c_void) };
            let stack_limit = unsafe { *((teb as *const u8).add(0x10) as *const *mut c_void) };
            let _ = tx_a.send((teb, stack_base as usize, stack_limit as usize));
            // BUSY-spin (no kernel wait) on the go flag until the test thread
            // releases it AFTER encrypt(). Spinning keeps the thread Running (not
            // kernel-waiting) so the fault it later takes is from RUNNING code
            // (read_volatile) — the context the VEH is built to recover. This test
            // never runs the watchdog/dormancy protocol, so a spinning thread is
            // harmless here (test-only).
            while !VEH_TEST_GO.load(Ordering::Acquire) {
                core::hint::spin_loop();
            }
            let r = unsafe { core::ptr::read_volatile(pa as *const u8) };
            VEH_TEST_RES_A.store(if r == expected_a { 1 } else { 2 }, Ordering::Release);
        });

        let tx_b = bounds_tx.clone();
        let _ = std::thread::spawn(move || {
            let teb: usize = unsafe {
                let t: usize;
                core::arch::asm!("mov {0}, gs:[0x30]", out(reg) t, options(nostack));
                t
            };
            let stack_base = unsafe { *((teb as *const u8).add(0x08) as *const *mut c_void) };
            let stack_limit = unsafe { *((teb as *const u8).add(0x10) as *const *mut c_void) };
            let _ = tx_b.send((teb, stack_base as usize, stack_limit as usize));
            while !VEH_TEST_GO.load(Ordering::Acquire) {
                core::hint::spin_loop();
            }
            let r = unsafe { core::ptr::read_volatile(pb as *const u8) };
            VEH_TEST_RES_B.store(if r == expected_b { 1 } else { 2 }, Ordering::Release);
        });

        // Collect both spawned threads' bounds, exclude their whole stacks +
        // TEB/PEB/TLS (so KiUserExceptionDispatcher can run on the faulting
        // thread), then the test thread's own, then encrypt.
        let (teb_a, sb_a, sl_a) = bounds_rx.recv().expect("thread A bounds");
        let (teb_b, sb_b, sl_b) = bounds_rx.recv().expect("thread B bounds");
        assert!(
            crate::memguard::exclude_thread_state_and_stack(
                &mut guard,
                teb_a as *const u8,
                sb_a as *const u8,
                sl_a as *const u8,
            ),
            "exclude spawned thread A failed"
        );
        assert!(
            crate::memguard::exclude_thread_state_and_stack(
                &mut guard,
                teb_b as *const u8,
                sb_b as *const u8,
                sl_b as *const u8,
            ),
            "exclude spawned thread B failed"
        );
        // Exclude the test thread's own stack/TEB/PEB/TLS before encrypting, or
        // the test thread's live stack would be encrypted + NOACCESS and the next
        // stack touch (incl. VEH dispatch) would fault -> APPCRASH.
        assert!(
            crate::memguard::exclude_current_thread(&mut guard),
            "exclude_current_thread failed"
        );
        // Encrypt the whole process (buf_a/buf_b included; spawned stacks/TEBs
        // excepted).
        assert!(guard.encrypt(), "encrypt failed");
        assert!(
            guard.encrypted_count() > 0,
            "encrypt should have produced encrypted pages"
        );

        // Simulate a watchdog crash mid-bulk-decrypt: decrypt nothing, leaving
        // every page (including buf_a/buf_b) NOACCESS + encrypted. The VEH is the
        // only thing that can recover them now.
        crate::memguard::debug_simulate_crash_partial_decrypt(0);

        // Release the spawned threads by flipping the go flag (no kernel event).
        // They are BUSY-spinning (Running), so this does not wake a kernel wait;
        // each then touches its encrypted page and is lazily decrypted by the VEH.
        VEH_TEST_GO.store(true, Ordering::Release);
        // BUSY-spin-wait on the result statics (no kernel wait, no logging while
        // memory is encrypted - only the non-allocating trace path is used).
        let start = unsafe { GetTickCount64() };
        loop {
            let ra = VEH_TEST_RES_A.load(Ordering::Acquire);
            let rb = VEH_TEST_RES_B.load(Ordering::Acquire);
            if ra != 0 && rb != 0 {
                break;
            }
            if unsafe { GetTickCount64() }.wrapping_sub(start) > 30_000 {
                panic!("VEH recovery did not complete within 30s (ra={ra} rb={rb})");
            }
            core::hint::spin_loop();
        }

        // The rest of the encrypted pages must still be recoverable via a normal
        // bulk decrypt. The VEH-decrypted pages are recorded PG_PLAINTEXT and
        // skipped, so they are never double-XOR'd back into ciphertext.
        assert!(guard.decrypt(), "decrypt after VEH recovery failed");
        assert_eq!(
            guard.encrypted_count(),
            0,
            "all pages should be plaintext after decrypt"
        );
        // VEH must have recovered both pages: result code 1 = ok (value matched).
        // Asserted after full decrypt (plaintext) to avoid any panic path while
        // memory is still encrypted.
        assert_eq!(
            VEH_TEST_RES_A.load(Ordering::Acquire),
            1,
            "thread A VEH recovery"
        );
        assert_eq!(
            VEH_TEST_RES_B.load(Ordering::Acquire),
            1,
            "thread B VEH recovery"
        );

        // Subsequent normal cycles must still work end to end.
        assert!(guard.encrypt(), "second encrypt failed");
        assert!(guard.decrypt(), "second decrypt failed");
        assert_eq!(guard.encrypted_count(), 0, "cycle should end decrypted");

        unsafe {
            let mut dec_a = [0u8; PAGE_SIZE];
            let mut dec_b = [0u8; PAGE_SIZE];
            core::ptr::copy_nonoverlapping(buf_a, dec_a.as_mut_ptr(), PAGE_SIZE);
            core::ptr::copy_nonoverlapping(buf_b, dec_b.as_mut_ptr(), PAGE_SIZE);
            for i in 0..PAGE_SIZE {
                assert_eq!(dec_a[i], pattern_a[i], "page A not restored (idx {i})");
                assert_eq!(dec_b[i], pattern_b[i], "page B not restored (idx {i})");
            }
            VirtualFree(buf_a as *mut c_void, 0, MEM_RELEASE);
            VirtualFree(buf_b as *mut c_void, 0, MEM_RELEASE);
        }
    }
}
