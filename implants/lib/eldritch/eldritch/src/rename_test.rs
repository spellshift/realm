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

#[test]
fn test_bytes_and_strings_interchangeable_in_methods() {
    let printer = Arc::new(BufferPrinter::new());
    let mut interp = Interpreter::new_with_printer(printer.clone())
        .with_agent(Arc::new(eldritch_mockagent::MockAgent::new()));
    interp.register_lib(crate::aes::std::StdAesLibrary);
    interp.register_lib(crate::base64::std::StdBase64Library);
    interp.register_lib(crate::file::std::StdFileLibrary);
    interp.register_lib(crate::gzip::std::StdGzipLibrary);
    interp.register_lib(crate::hash::std::StdHashLibrary);
    interp.register_lib(crate::json::std::StdJsonLibrary);

    // 1. print(b"stuff") and eprint(b"stuff")
    printer.clear();
    interp.interpret(r#"print(b"stuff")"#).unwrap();
    assert_eq!(printer.read_out(), "stuff");
    assert_eq!(printer.read_err(), "");

    printer.clear();
    interp.interpret(r#"eprint(b"stuff")"#).unwrap();
    assert_eq!(printer.read_out(), "");
    assert_eq!(printer.read_err(), "stuff");

    // 2. file.write and file.append accept both strings and bytes
    let directory = tempfile::tempdir().unwrap();
    let dst = directory.path().join("dst.txt");
    interp.define_variable(
        "dst_path",
        crate::Value::String(dst.to_string_lossy().into_owned()),
    );
    interp.define_variable(
        "dst_path_bytes",
        crate::Value::Bytes(dst.to_string_lossy().as_bytes().to_vec()),
    );

    // Write string, append bytes, append string
    interp
        .interpret(r#"file.write(dst_path, "my string")"#)
        .unwrap();
    assert_eq!(std::fs::read(&dst).unwrap(), b"my string");

    interp
        .interpret(r#"file.append(dst_path, " and more string")"#)
        .unwrap();
    assert_eq!(std::fs::read(&dst).unwrap(), b"my string and more string");

    interp
        .interpret(r#"file.append(dst_path, b" and bytes")"#)
        .unwrap();
    assert_eq!(
        std::fs::read(&dst).unwrap(),
        b"my string and more string and bytes"
    );

    // Write bytes using bytes path, read back using both str and bytes path
    interp
        .interpret(r#"file.write(dst_path_bytes, b"fresh bytes")"#)
        .unwrap();
    assert_eq!(
        interp.interpret("file.read(dst_path)").unwrap(),
        crate::Value::Bytes(b"fresh bytes".to_vec())
    );
    assert_eq!(
        interp.interpret("file.read(dst_path_bytes)").unwrap(),
        crate::Value::Bytes(b"fresh bytes".to_vec())
    );

    // Print file.read(dst_path) directly
    printer.clear();
    interp.interpret("print(file.read(dst_path))").unwrap();
    assert_eq!(printer.read_out(), "fresh bytes");

    // 3. Other stdlib methods accept both bytes and strings
    assert_eq!(
        interp.interpret(r#"hash.md5("hello")"#).unwrap(),
        interp.interpret(r#"hash.md5(b"hello")"#).unwrap()
    );
    assert_eq!(
        interp.interpret(r#"gzip.compress("hello")"#).unwrap(),
        interp.interpret(r#"gzip.compress(b"hello")"#).unwrap()
    );
    assert_eq!(
        interp.interpret(r#"json.decode('{"k": 1}')"#).unwrap(),
        interp.interpret(r#"json.decode(b'{"k": 1}')"#).unwrap()
    );
    assert_eq!(
        interp.interpret(r#"base64.encode("hello")"#).unwrap(),
        interp.interpret(r#"base64.encode(b"hello")"#).unwrap()
    );
    assert_eq!(
        interp
            .interpret(r#"aes.decrypt("0123456789abcdef", "fedcba9876543210", aes.encrypt("0123456789abcdef", "fedcba9876543210", "secret message"))"#)
            .unwrap(),
        crate::Value::Bytes(b"secret message".to_vec())
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
    interp.register_lib(crate::registry::fake::RegistryLibraryFake);
    interp.register_lib(crate::sys::fake::SysLibraryFake);
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

    printer.clear();
    let expected_read = interp
        .interpret("registry.read('HKCU\\\\Software')")
        .unwrap();
    let actual_read = interp.interpret("sys.get_reg('HKCU\\\\Software')").unwrap();
    assert_eq!(actual_read, expected_read);
    assert_eq!(
        printer.read_err(),
        "sys.get_reg has been deprecated and will be removed in a future version of eldritch. Please use registry.read instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."
    );

    printer.clear();
    let expected_write = interp
        .interpret("registry.write('HKCU\\\\Software', 'Test', 'REG_SZ', 'val')")
        .unwrap();
    let actual_write = interp
        .interpret("sys.write_reg('HKCU\\\\Software', 'Test', 'REG_SZ', 'val')")
        .unwrap();
    assert_eq!(actual_write, expected_write);
    assert_eq!(
        printer.read_err(),
        "sys.write_reg has been deprecated and will be removed in a future version of eldritch. Please use registry.write instead. You can also use our eldritch_migrate_0.4.1.py script to automatically update."
    );
}
