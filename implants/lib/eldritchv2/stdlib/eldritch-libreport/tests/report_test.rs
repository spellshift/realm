use eldritch_core::Interpreter;
use eldritch_libreport::std::StdReportLibrary;
use eldritch_libagent::fake::AgentFake;
use std::sync::Arc;
use tempfile::NamedTempFile;
use std::io::Write;

#[test]
fn test_report_bindings() {
    let agent = Arc::new(AgentFake::default());
    let lib = StdReportLibrary::new(agent, 123);
    let mut interp = Interpreter::new();
    interp.register_lib(lib);

    // report.user_password
    let res = interp.interpret("report.user_password('user', 'pass')");
    assert!(res.is_ok());

    // report.ssh_key
    let res = interp.interpret("report.ssh_key('user', 'key')");
    assert!(res.is_ok());

    // report.process_list
    // The parser error was due to indentation in raw string literal.
    let code = r#"
plist = [dict(pid=1, name="init", ppid=0, path="/sbin/init", cmd="init", cwd="/", env=dict(), principal="root")]
report.process_list(plist)
"#;
    let res = interp.interpret(code);
    if res.is_err() {
        println!("Error: {:?}", res.as_ref().err());
    }
    assert!(res.is_ok());

    // report.file
    let mut temp = NamedTempFile::new().unwrap();
    write!(temp, "secret").unwrap();
    let path = temp.path().to_str().unwrap().replace("\\", "/");

    let code = format!("report.file('{}')", path);
    let res = interp.interpret(&code);
    assert!(res.is_ok());
}
