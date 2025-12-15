use alloc::string::String;
use anyhow::Result;
use crate::std::StdPivotLibrary;

pub fn run(lib: &StdPivotLibrary, cmd: Option<String>) -> Result<(), String> {
    // Current `start_reverse_shell` method is in `eldritch_agent::Agent`.
    // We just call it. It should be async or fire-and-forget?
    // Let's check `eldritch-agent` definition provided earlier.
    // It has: `fn start_reverse_shell(&self, task_id: i64, cmd: Option<String>) -> Result<(), String>;`
    // And `reverse_shell`? `fn reverse_shell(&self) -> Result<(), String>;` (Removed from library trait but likely exists on Agent).

    // The previous implementation used `start_reverse_shell`.
    lib.agent
        .start_reverse_shell(lib.task_id, cmd)
}
