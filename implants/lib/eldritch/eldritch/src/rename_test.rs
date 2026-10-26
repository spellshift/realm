use crate::{BufferPrinter, Interpreter};
use std::sync::Arc;

fn check_aliases(interp: &mut Interpreter, printer: &BufferPrinter) {
    for (old, new, args) in [
        ("file.temp_file", "file.tmp", "'eldritch-rename-test.txt'"),
        (
            "time.format_to_epoch",
            "time.to_epoch",
            "'2021-01-01 00:00:00', '%Y-%m-%d %H:%M:%S'",
        ),
        ("time.format_to_readable", "time.format", "1609459200, '%Y'"),
        ("agent.get_config", "agent.config", ""),
        ("agent.get_transport", "agent.transport", ""),
        ("agent.get_callback_interval", "agent.callback_interval", ""),
        ("file.pwd", "process.cwd", ""),
        ("process.netstat", "net.conns", ""),
        (
            "file.template_str",
            "template.render",
            "'Hello {{ name }}', {'name': 'World'}, True",
        ),
        (
            "file.template",
            "template.render_file",
            "template_source, template_output, {'name': 'World'}, True",
        ),
    ] {
        printer.clear();
        let expected = interp.interpret(&format!("{new}({args})")).unwrap();
        assert_eq!(printer.read_err(), "", "{new} should not warn");
        let actual = interp.interpret(&format!("{old}({args})")).unwrap();
        assert_eq!(actual, expected, "{old} must delegate to {new}");
        assert_eq!(
            printer.read_err(),
            format!(
                "{old} has been deprecated and will be removed in a future version of eldritch. Please use {new} instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."
            ),
        );
        assert_eq!(printer.read_out(), "");
    }

    printer.clear();
    let expected_enc = interp.interpret("'hello'.encode('utf16le')").unwrap();
    let actual_enc = interp.interpret("crypto.encode_utf16le('hello')").unwrap();
    assert_eq!(actual_enc, expected_enc);
    assert_eq!(
        printer.read_err(),
        "crypto.encode_utf16le has been deprecated and will be removed in a future version of eldritch. Please use str.encode(\"utf16le\") instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."
    );

    printer.clear();
    let expected_dec = interp
        .interpret("'hello'.encode('utf16le').decode('utf16le')")
        .unwrap();
    let actual_dec = interp
        .interpret("crypto.decode_utf16le('hello'.encode('utf16le'))")
        .unwrap();
    assert_eq!(actual_dec, expected_dec);
    assert_eq!(
        printer.read_err(),
        "crypto.decode_utf16le has been deprecated and will be removed in a future version of eldritch. Please use bytes.decode(\"utf16le\") instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."
    );
}

#[test]
fn test_std_renames_delegate_and_warn() {
    let printer = Arc::new(BufferPrinter::new());
    let mut interp = Interpreter::new_with_printer(printer.clone())
        .with_agent(Arc::new(eldritch_mockagent::MockAgent::new()));
    interp.register_lib(crate::crypto::std::StdCryptoLibrary);
    interp.register_lib(crate::file::std::StdFileLibrary);
    interp.register_lib(crate::net::std::StdNetLibrary);
    interp.register_lib(crate::process::std::StdProcessLibrary);
    interp.register_lib(crate::time::std::StdTimeLibrary);
    interp.register_lib(crate::template::std::StdTemplateLibrary);
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("source.j2");
    let output = directory.path().join("output.txt");
    std::fs::write(&source, "Hello {{ name }}").unwrap();
    interp.define_variable(
        "template_source",
        crate::Value::String(source.to_string_lossy().into_owned()),
    );
    interp.define_variable(
        "template_output",
        crate::Value::String(output.to_string_lossy().into_owned()),
    );
    check_aliases(&mut interp, &printer);
    assert_eq!(std::fs::read_to_string(output).unwrap(), "Hello World");
    assert_eq!(
        interp
            .interpret("template.render('{{ value }}', {'value': '<realm>'}, True)")
            .unwrap(),
        crate::Value::String("&lt;realm&gt;".into()),
    );
    assert!(
        interp
            .interpret("template.render('{% if', {}, False)")
            .is_err()
    );
}

#[cfg(feature = "fake_bindings")]
#[test]
fn test_fake_renames_delegate_and_warn() {
    let printer = Arc::new(BufferPrinter::new());
    let mut interp = Interpreter::new_with_printer(printer.clone());
    interp.register_lib(crate::agent::fake::AgentLibraryFake);
    interp.register_lib(crate::crypto::fake::CryptoLibraryFake);
    interp.register_lib(crate::file::fake::FileLibraryFake::default());
    interp.register_lib(crate::net::fake::NetLibraryFake);
    interp.register_lib(crate::process::fake::ProcessLibraryFake);
    interp.register_lib(crate::time::fake::TimeLibraryFake);
    interp.register_lib(crate::template::fake::TemplateLibraryFake);
    interp.define_variable(
        "template_source",
        crate::Value::String("/tmp/source".into()),
    );
    interp.define_variable(
        "template_output",
        crate::Value::String("/tmp/output".into()),
    );
    check_aliases(&mut interp, &printer);
}
