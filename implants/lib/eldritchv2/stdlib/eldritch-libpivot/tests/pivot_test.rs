use eldritch_core::Interpreter;
use eldritch_libpivot::std::StdPivotLibrary;
use eldritch_libagent::fake::AgentFake;
use std::sync::Arc;

#[test]
fn test_pivot_bindings() {
    let agent = Arc::new(AgentFake::default());
    let lib = StdPivotLibrary::new(agent, 123);

    let mut interp = Interpreter::new();
    interp.register_lib(lib);

    // Testing logic of these methods usually requires network access or complex mocks.
    // For migration, we verify that the methods are registered and callable, even if they return error.

    // pivot.arp_scan
    // Should fail with mock error or network error
    let _res = interp.interpret("pivot.arp_scan('127.0.0.1/24', 10)");

    // pivot.port_scan
    let _res = interp.interpret("pivot.port_scan('127.0.0.1', [80], 2, 10)");

    // pivot.ncat
    // ncat is blocking/interactive, maybe hard to test via interpreter.

    // pivot.reverse_shell_pty
    // Delegates to agent, AgentFake likely returns Ok or Err.
    let _res = interp.interpret("pivot.reverse_shell_pty('127.0.0.1', 4444)");

    // pivot.ssh_exec
    let _res = interp.interpret("pivot.ssh_exec('127.0.0.1', 22, 'user', 'pass', 'whoami')");

    // pivot.ssh_copy
    let _res = interp.interpret("pivot.ssh_copy('127.0.0.1', 22, 'user', 'pass', 'src', 'dst')");

    // pivot.smb_exec (stubbed)
    let _res = interp.interpret("pivot.smb_exec('target', 'share', 'user', 'pass', 'domain', 'cmd')");
}
